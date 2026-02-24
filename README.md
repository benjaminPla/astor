# tsu

> A fast, minimal HTTP framework for Rust applications deployed behind a reverse proxy.

**tsu** (tiramisu 🍰) is built on the same foundation as [axum] — tokio + hyper — but with a narrower, opinionated focus: applications that live behind **nginx** or a **Kubernetes ingress** and don't need the framework to re-implement concerns the reverse proxy already handles.

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
| Async I/O | tokio + hyper 1 |
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
        .get("/", hello)
        .get("/users/:id", get_user)
        .get("/healthz", health::liveness)   // k8s liveness probe
        .get("/readyz",  health::readiness); // k8s readiness probe

    Server::bind("0.0.0.0:3000")
        .serve(app)
        .await
        .unwrap();
}

async fn hello(_req: Request) -> Response {
    Response::text("Hello!")
}

async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    Response::json(format!(r#"{{"id":"{id}"}}"#))
}
```

---

## Routing

Routes use `:param` syntax for path parameters:

```rust
router
    .get("/users/:id",              get_user)
    .get("/orgs/:org/repos/:repo",  get_repo)
    .post("/users",                 create_user)
    .delete("/users/:id",           delete_user)
    .patch("/users/:id",            update_user);
```

Retrieve parameters inside the handler:

```rust
async fn get_repo(req: Request) -> Response {
    let org  = req.param("org").unwrap();
    let repo = req.param("repo").unwrap();
    Response::text(format!("{org}/{repo}"))
}
```

---

## Custom response types

Implement `IntoResponse` to return your own types directly:

```rust
use tsu::{IntoResponse, Response};

struct Json<T: serde::Serialize>(T);

impl<T: serde::Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        Response::json(serde_json::to_string(&self.0).unwrap())
    }
}

// Now you can use it as a handler return type:
async fn handler(_req: Request) -> Json<MyStruct> {
    Json(MyStruct { … })
}
```

---

## Health checks

```rust
use tsu::{Router, health};

let app = Router::new()
    .get("/healthz", health::liveness)   // always 200
    .get("/readyz",  health::readiness); // 200 when ready
```

Override readiness to gate on dependency health:

```rust
use http::StatusCode;

async fn readiness(_req: Request) -> Response {
    if db_pool_is_healthy().await {
        Response::text("ready")
    } else {
        Response::status(StatusCode::SERVICE_UNAVAILABLE)
    }
}
```

---

## Deployment

### Local development

```sh
RUST_LOG=info cargo run --example basic
curl http://localhost:3000/
```

### With nginx

See [`nginx/nginx.conf`](nginx/nginx.conf) for a production-ready configuration.

Key settings:

```nginx
proxy_http_version 1.1;
proxy_set_header   Connection "";   # enables keep-alive to the backend
client_max_body_size 10m;           # enforced by nginx, not tsu

# REQUIRED — do not set to off.
# tsu only reads Content-Length-framed bodies. proxy_buffering on (the nginx
# default) guarantees nginx buffers the full request body and forwards it with
# Content-Length. Setting it to off allows chunked bodies that tsu cannot parse.
proxy_buffering on;
```

### On Kubernetes

See the manifests in [`k8s/`](k8s/):

| File | Purpose |
|---|---|
| `deployment.yaml` | Pod spec with probes and `terminationGracePeriodSeconds` |
| `service.yaml` | ClusterIP service on port 3000 |
| `ingress.yaml` | ingress-nginx with TLS, body-size, and keepalive annotations |

**Required:** set `terminationGracePeriodSeconds` to a value longer than your
slowest request so tsu has time to drain in-flight work before SIGKILL.

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

[axum]: https://github.com/tokio-rs/axum
[matchit]: https://github.com/ibraheemdev/matchit
[tracing]: https://github.com/tokio-rs/tracing
