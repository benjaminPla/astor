# tsu

> A fast, minimal HTTP framework for Rust applications deployed behind a reverse proxy.

**tsu** is built on tokio with a narrow, opinionated focus: applications that live behind **nginx** or a **Kubernetes ingress** and don't need the framework to re-implement concerns the reverse proxy already handles.

---

## The reverse-proxy contract

When nginx (or ingress-nginx) sits in front of your service, it already covers:

| Concern | Where |
|---|---|
| TLS termination | nginx SSL / k8s ingress TLS |
| Body-size limits | `client_max_body_size` in nginx |
| Rate limiting | `limit_req` / ingress-nginx annotations |
| Slow-client & DDoS protection | nginx timeouts and buffers |
| HTTP/2 & HTTP/3 to clients | nginx upstream negotiation |

**tsu** owns the rest:

| Concern | How |
|---|---|
| Radix-tree routing | [`matchit`] — O(path-length) lookup |
| Async I/O | tokio |
| Graceful shutdown | SIGTERM + Ctrl-C; waits for in-flight requests |
| Health probes | Built-in `/healthz` and `/readyz` |
| Structured logging | [`tracing`] crate |

---

## Quick start

```toml
# Cargo.toml
[dependencies]
tsu   = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use tsu::{Router, Server, Request, Response, health};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .get("/users/:id", get_user)
        .get("/healthz",   health::liveness)
        .get("/readyz",    health::readiness);

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
    .get("/users/:id",             get_user)
    .get("/orgs/:org/repos/:repo", get_repo)
    .post("/users",                create_user)
    .delete("/users/:id",          delete_user)
    .patch("/users/:id",           update_user)
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

All status codes go through `Status`. Every IANA-registered code is a variant:

```rust
use tsu::Status;

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

### Shortcuts — `200 OK`, no custom headers

```rust
use tsu::{Response, Status};

// JSON — pass bytes from your serialiser directly, zero extra allocation.
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

The builder always terminates with a typed body method. You always know what you're sending.

```rust
use tsu::{Response, ContentType, Status};

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
| `ContentType::Json` | `application/json` |
| `ContentType::Text` | `text/plain; charset=utf-8` |
| `ContentType::Html` | `text/html; charset=utf-8` |
| `ContentType::Xml` | `application/xml` |
| `ContentType::OctetStream` | `application/octet-stream` |
| `ContentType::FormData` | `application/x-www-form-urlencoded` |
| `ContentType::EventStream` | `text/event-stream` |
| `ContentType::Csv` | `text/csv` |
| `ContentType::Pdf` | `application/pdf` |
| `ContentType::MsgPack` | `application/msgpack` |

### Reading request bodies

`req.body()` returns `&[u8]`. Parse it however you want — tsu never touches the bytes:

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

Implement `IntoResponse` on your own types to return them directly from handlers without constructing `Response` manually every time:

```rust
use tsu::{IntoResponse, Response, Status};
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
// Return Status directly from a handler — no Response construction needed
async fn delete_user(_req: Request) -> Status { Status::NoContent }
```

---

## Health checks

```rust
use tsu::{Router, health};

let app = Router::new()
    .get("/healthz", health::liveness)   // always 200 — is the process alive?
    .get("/readyz",  health::readiness); // 200 when ready to serve traffic
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

**How keep-alive works:**

```
client ──(h2/h1.1)──► nginx ──(HTTP/1.1 keep-alive pool)──► tsu
```

nginx maintains a pool of idle TCP connections to tsu. Requests are served
over those connections without a TCP handshake per request. tsu loops on each
connection until nginx closes it — tsu never manages connection lifetime itself.

**Required proxy settings:**

```nginx
proxy_http_version 1.1;
proxy_set_header   Connection "";   # clears nginx's default "close", enabling keep-alive
client_max_body_size 10m;           # enforced by nginx, not tsu

# REQUIRED — do not set to off.
# tsu only reads Content-Length-framed bodies. proxy_buffering on (the default)
# guarantees nginx buffers the full request body before forwarding it.
proxy_buffering on;
```

**Tuning the upstream connection pool** (in the `upstream` block):

```nginx
upstream tsu_backend {
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
| `service.yaml` | ClusterIP service on port 3000 |
| `ingress.yaml` | ingress-nginx with TLS, body-size, and keepalive annotations |

**Required:** set `terminationGracePeriodSeconds` longer than your slowest request
so tsu has time to drain in-flight work before SIGKILL.

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

## License

MIT

[matchit]: https://github.com/ibraheemdev/matchit
[tracing]: https://github.com/tokio-rs/tracing
