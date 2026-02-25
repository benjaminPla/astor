# astor

[![Crates.io](https://img.shields.io/crates/v/astor)](https://crates.io/crates/astor)
[![docs.rs](https://img.shields.io/docsrs/astor)](https://docs.rs/astor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/benjaminPla/astor/actions/workflows/ci.yml/badge.svg)](https://github.com/benjaminPla/astor/actions)

> Minimal HTTP framework for Rust. Lives behind nginx. Does its job. Goes home.

Your nginx handles TLS.
Your nginx handles rate limiting.
Your nginx handles slow clients, body sizes, and half the other things frameworks love to re-implement.

**So what exactly is your framework supposed to duplicate?**

Nothing. astor doesn't touch any of that. The proxy does proxy things. The framework does framework things. This is not a controversial opinion.

---

## The deal

astor sits behind nginx or ingress-nginx. The proxy covers the hard, boring, already-solved stuff. astor covers your routes.

What the proxy already owns — and why we sleep soundly knowing it:

| nginx / ingress handles this | what astor thinks about it |
|---|---|
| Body-size limits | `client_max_body_size` in nginx. Done. |
| HTTP/2 + HTTP/3 to clients | nginx negotiates protocol. astor speaks plain HTTP/1.1. |
| Rate limiting | `limit_req` or ingress-nginx annotations. Not our concern. |
| Slow-client & DDoS protection | nginx timeouts and buffers. We trust nginx. |
| TLS termination | nginx SSL / k8s ingress TLS. Obviously. |

What's left for astor — which is, coincidentally, the only part that changes between applications:

| What | How |
|---|---|
| Async I/O | tokio |
| Graceful shutdown | SIGTERM + Ctrl-C — drains in-flight requests before exit |
| Health probes | `/healthz` and `/readyz` built in |
| Radix-tree routing | [`matchit`] — O(path-length) lookup |

## Dependencies

astor keeps its dependency tree minimal by design. It speaks raw HTTP/1.1 over tokio — no hyper, no tower, no middleware stack you didn't ask for.

Every crate that lives in `[dependencies]` is there because the alternative is re-implementing it badly. Everything else is your problem, not ours. You want logging? Bring your own. You want tracing? Wire it up in your app. astor surfaces errors as `Result` — catch them, log them however you like.

---

## Quick start

```toml
# Cargo.toml
[dependencies]
astor   = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use astor::{health, Method, Request, Response, Router, Server};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .on(Method::Get, "/healthz",   health::liveness)
        .on(Method::Get, "/readyz",    health::readiness)
        .on(Method::Get, "/users/:id", get_user);

    Server::bind("0.0.0.0:3000").serve(app).await.unwrap();
}

async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    Response::json(format!(r#"{{"id":"{id}"}}"#).into_bytes())
}
```

---

## Routing

```rust
use astor::Method;

router
    .on(Method::Delete,  "/users/:id",          delete_user)
    .on(Method::Get,     "/users/:id",          get_user)
    .on(Method::Get,     "/orgs/:org/repos/:repo", get_repo)
    .on(Method::Options, "/users",              options_users)
    .on(Method::Patch,   "/users/:id",          update_user)
    .on(Method::Post,    "/users",              create_user);
```

Path parameters via `req.param("name")`:

```rust
async fn get_repo(req: Request) -> Response {
    let org  = req.param("org").unwrap();
    let repo = req.param("repo").unwrap();
    Response::text(format!("{org}/{repo}"))
}
```

---

## Responses

### Status codes

All status codes go through `Status`. Every IANA-registered code is a named variant — no magic integers:

```rust
use astor::Status;

Status::Ok                     // 200
Status::Created                // 201
Status::NoContent              // 204
Status::BadRequest             // 400
Status::Unauthorized           // 401
Status::NotFound               // 404
Status::UnprocessableContent   // 422
Status::TooManyRequests        // 429
Status::InternalServerError    // 500
Status::ServiceUnavailable     // 503
```

### Shortcuts — `200 OK`, no custom headers needed

```rust
use astor::{Response, Status};

// JSON — bytes from your serialiser, directly. No intermediate allocation.
// serde_json:  Response::json(serde_json::to_vec(&val).unwrap())
// hand-built:  Response::json(format!(r#"{{"id":{id}}}"#).into_bytes())
Response::json(bytes)

// Plain text
Response::text("hello")

// No body
Response::status(Status::NoContent)
Response::status(Status::NotFound)
```

### Builder — custom status or extra headers

Ends with a typed body call. You always know exactly what you're sending.

```rust
use astor::{Response, ContentType, Status};

// 201 Created with Location header, JSON body
Response::builder()
    .status(Status::Created)
    .header("location", "/users/42")
    .json(bytes)

// 301 redirect — no body
Response::builder()
    .status(Status::MovedPermanently)
    .header("location", "/new-path")
    .no_body()

// Any content-type via the ContentType enum
Response::builder()
    .status(Status::Ok)
    .bytes(ContentType::Xml, b"<users/>".to_vec())
```

### ContentType enum

| Variant | Content-Type header |
|---|---|
| `ContentType::Csv` | `text/csv` |
| `ContentType::EventStream` | `text/event-stream` |
| `ContentType::FormData` | `application/x-www-form-urlencoded` |
| `ContentType::Html` | `text/html; charset=utf-8` |
| `ContentType::Json` | `application/json` |
| `ContentType::MsgPack` | `application/msgpack` |
| `ContentType::OctetStream` | `application/octet-stream` |
| `ContentType::Pdf` | `application/pdf` |
| `ContentType::Text` | `text/plain; charset=utf-8` |
| `ContentType::Xml` | `application/xml` |

### Reading request bodies

`req.body()` returns `&[u8]`. Parse it however you want — astor never touches the bytes:

```rust
async fn create_user(req: Request) -> Response {
    if req.body().is_empty() {
        return Response::status(Status::BadRequest);
    }
    let user: User = serde_json::from_slice(req.body()).unwrap();
    Response::json(serde_json::to_vec(&user).unwrap())
}
```

---

## Custom return types with `IntoResponse`

Implement `IntoResponse` on your own types and return them directly from handlers. No `Response` construction scattered across every call site:

```rust
use astor::{IntoResponse, Response, Status};
use serde::Serialize;

struct Json<T: Serialize>(T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(bytes) => Response::json(bytes),
            Err(_)    => Response::status(Status::InternalServerError),
        }
    }
}

// Handler return type is inferred — no Response construction at the call site.
async fn get_user(_req: Request) -> Json<User> {
    Json(User { id: 1, name: "alice".into() })
}
```

Built-in `IntoResponse` impls: `Response`, `String`, `&'static str`, `Status`.

```rust
// Return Status directly — astor wraps it. No boilerplate.
async fn delete_user(_req: Request) -> Status { Status::NoContent }
```

---

## Health checks

Kubernetes needs to know if your pod is alive and ready. Two endpoints. Always 200 if the process can respond. That's it.

```rust
use astor::{Router, health};

let app = Router::new()
    .get("/healthz", health::liveness)   // is the process alive?
    .get("/readyz",  health::readiness); // ready to serve traffic?
```

Custom readiness to gate on dependency health:

```rust
async fn readiness(_req: Request) -> Response {
    if db_pool_is_healthy().await {
        Response::text("ready")
    } else {
        Response::status(Status::ServiceUnavailable)
    }
}
```

---

## Deployment

### Local development

```sh
cargo run --example basic
curl http://localhost:3000/users/42
```

### With nginx

```
client ──(h2/h1.1)──► nginx ──(HTTP/1.1 keep-alive pool)──► astor
```

nginx handles TLS, rate limiting, slow clients, and body-size limits. astor does not — by design. The configuration below is what makes that contract work.

#### Keep-alive pool

nginx reuses TCP connections to astor instead of opening a new one per request. astor loops on each connection until nginx closes it — it never inspects the `Connection` header.

```nginx
upstream astor_backend {
    server 127.0.0.1:3000;

    keepalive 64;            # idle connections per worker
    keepalive_requests 1000; # recycle after N requests
    keepalive_timeout  60s;  # close idle connections after this long
}
```

Rule of thumb for `keepalive`: `(expected RPS / nginx workers) × avg request duration (s)`. Too low → TCP churn under load. Too high → idle file descriptors accumulate.

#### Required location block

```nginx
location / {
    # nginx forwards ALL methods by default — including unknown garbage.
    # List every method your app uses. Everything else gets 405.
    limit_except GET POST PUT PATCH DELETE OPTIONS HEAD CONNECT TRACE {
        return 405;
    }

    proxy_pass         http://astor_backend;

    # Required for keep-alive: HTTP/1.1 + clear the default "close" header.
    proxy_http_version 1.1;
    proxy_set_header   Connection  "";

    # astor reads Content-Length-framed bodies only.
    # Do not set proxy_buffering off — chunked bodies will not be read.
    proxy_buffering    on;

    # Body size, slow-client protection, and rate limiting belong here —
    # nginx enforces them before the request reaches astor.
    client_max_body_size  10m;
    client_body_timeout   30s;
    client_header_timeout 10s;
}
```

### On Kubernetes

ingress-nginx is an nginx instance — the same rules above apply. Set the equivalent annotations on your `Ingress` resource.

The one astor-specific k8s requirement is `terminationGracePeriodSeconds`. On `SIGTERM`, astor stops accepting new connections and drains in-flight requests before exiting. If this value is shorter than your slowest request, k8s sends `SIGKILL` while requests are still running — that is not graceful shutdown.

```yaml
spec:
  terminationGracePeriodSeconds: 30  # must be longer than your slowest request
  containers:
    - name: app
      image: your-registry/your-app:latest
      livenessProbe:
        httpGet: { path: /healthz, port: 3000 }
      readinessProbe:
        httpGet: { path: /readyz, port: 3000 }
```

---

## A note on ordering

Everything in this codebase that can be alphabetically ordered, is. Enum variants. Function parameters. Struct fields. Imports. Table rows. Everything.

This is not a stylistic preference. It is a rule. When things are ordered, you stop thinking about where they are and start thinking about what they do. You search for `Html` in the `ContentType` enum and your eye goes straight to the `H`s. You add a new variant and you know exactly where it lives. No debates, no "should this go before or after that" — alphabetical order is always the right answer and it is never wrong.

If you open a PR and something that could be alphabetically ordered is not, it will be sent back.

---

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR. See [CHANGELOG.md](CHANGELOG.md) for release history.

---

## License

MIT

[matchit]: https://github.com/ibraheemdev/matchit
