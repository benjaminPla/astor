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
| Structured logging | [`tracing`] crate |

---

## Quick start

```toml
# Cargo.toml
[dependencies]
astor   = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use astor::{Router, Server, Request, Response, health};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .get("/healthz",   health::liveness)
        .get("/readyz",    health::readiness)
        .get("/users/:id", get_user);

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
router
    .delete("/users/:id",          delete_user)
    .get("/users/:id",             get_user)
    .get("/orgs/:org/repos/:repo", get_repo)
    .patch("/users/:id",           update_user)
    .post("/users",                create_user)
    .route("OPTIONS", "/users",    options_users); // arbitrary method
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
RUST_LOG=info cargo run --example basic
curl http://localhost:3000/users/42
```

### With nginx

See [`nginx/nginx.conf`](nginx/nginx.conf) for a production-ready configuration.

**How keep-alive works — and why astor doesn't manage it:**

```
client ──(h2/h1.1)──► nginx ──(HTTP/1.1 keep-alive pool)──► astor
```

nginx maintains a pool of idle TCP connections to astor. Requests reuse those connections — no handshake per request. astor loops on each connection until nginx closes it. Connection lifetime is nginx's business. astor doesn't inspect the `Connection` header, and it never will.

**Required proxy settings:**

```nginx
proxy_http_version 1.1;
proxy_set_header   Connection "";   # clears nginx's default "close", enabling keep-alive
client_max_body_size 10m;           # enforced by nginx, not astor

# REQUIRED — do not set to off.
# astor only reads Content-Length-framed bodies. proxy_buffering on (the default)
# guarantees nginx buffers the full request body before forwarding it.
proxy_buffering on;
```

**Tuning the upstream connection pool** (in the `upstream` block):

```nginx
upstream astor_backend {
    server 127.0.0.1:3000;

    keepalive 64;            # idle connections per worker — raise if you see TCP churn
    keepalive_requests 1000; # recycle connection after N requests (default 1000)
    keepalive_timeout  60s;  # close idle connection after this long (default 60s)
}
```

Rule of thumb for `keepalive`: `(expected RPS / nginx workers) × avg request duration (s)`.
Too low → pool exhausts under load, new TCP connections are opened.
Too high → idle file descriptors accumulate across workers.

### On Kubernetes

See the manifests in [`k8s/`](k8s/):

| File | Purpose |
|---|---|
| `deployment.yaml` | Pod spec with probes and `terminationGracePeriodSeconds` |
| `ingress.yaml` | ingress-nginx with TLS, body-size, and keepalive annotations |
| `service.yaml` | ClusterIP service on port 3000 |

**Required:** set `terminationGracePeriodSeconds` longer than your slowest request. Otherwise k8s SIGKILLs the pod before astor finishes draining. That is not graceful shutdown.

```yaml
spec:
  terminationGracePeriodSeconds: 30  # adjust to your workload
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
[tracing]: https://github.com/tokio-rs/tracing
