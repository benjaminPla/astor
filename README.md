# astor

[![Crates.io](https://img.shields.io/crates/v/astor)](https://crates.io/crates/astor)
[![docs.rs](https://img.shields.io/docsrs/astor)](https://docs.rs/astor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/benjaminPla/astor/actions/workflows/ci.yml/badge.svg)](https://github.com/benjaminPla/astor/actions)

> HTTP for Rust services behind a reverse proxy. Does its job. Goes home.

Two dependencies — [`matchit`] for routing, `tokio` for async I/O. No hyper. No http crate. No middleware stack you didn't ask for. astor routes requests, builds typed responses, and stays out of every problem the proxy already solved.

---

## nginx handles this. astor doesn't.

astor is designed for the common deployment: your service lives behind nginx or ingress-nginx. The proxy already solved the hard problems. Re-implementing them in the framework is waste.

- **body size** — `client_max_body_size` in nginx ✓
- **header size** — `large_client_header_buffers` in nginx ✓
- **rate limiting** — `limit_req_zone` / ingress-nginx annotations ✓
- **slow clients** — `client_body_timeout`, `client_header_timeout` in nginx ✓
- **TLS** — nginx SSL / k8s ingress TLS ✓
- **HTTP/2 + HTTP/3 to clients** — nginx negotiates; astor speaks HTTP/1.1 upstream ✓

What astor covers — the only part that changes between applications:

- **Routing** — radix tree via [`matchit`], O(path-length) lookup, no allocations on the hot path
- **Async I/O** — raw tokio, no hyper
- **Graceful shutdown** — SIGTERM / Ctrl-C, drains in-flight requests before exit

---

## Quick start

```toml
# Cargo.toml
[dependencies]
astor = "0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use astor::{Method, Request, Response, Router, Server, Status};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .on(Method::Delete, "/users/{id}", delete_user)
        .on(Method::Get,    "/users/{id}", get_user)
        .on(Method::Post,   "/users",      create_user);

    Server::bind("0.0.0.0:3000").serve(app).await.unwrap();
}

// req.param("id") → Option<&str>. Path params use {name} syntax.
async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    Response::json(format!(r#"{{"id":"{id}"}}"#).into_bytes())
}

// req.body() → &[u8]. Parse with serde_json, simd-json, or anything else.
async fn create_user(req: Request) -> Response {
    if req.body().is_empty() {
        return Response::status(Status::BadRequest);
    }
    Response::builder()
        .status(Status::Created)
        .header("location", "/users/99")
        .json(r#"{"id":"99"}"#.to_owned().into_bytes())
}

// Return Status directly from a handler — astor wraps it into a response.
async fn delete_user(_req: Request) -> Status { Status::NoContent }
```

---

## Status codes are a type, not a number

astor has no free-form response constructor. You cannot pass a raw integer where a status code goes — the compiler stops you. Every status code is a named variant you can tab-complete, grep, and reason about.

```rust
use astor::{Response, Status};

// Named. Clear. Greppable. The compiler knows these are correct.
Response::status(Status::NoContent)   // 204 — not "204", not 204, not 20_4
Response::status(Status::NotFound)    // 404
Response::status(Status::Created)     // 201

// The builder enforces the same contract. Explicit at every step.
// Not response(201, bytes) — there are no magic integers here.
Response::builder()
    .status(Status::Created)
    .header("location", "/users/42")
    .json(bytes)

// Return Status directly from a handler — no Response construction needed.
async fn delete_user(_req: Request) -> Status { Status::NoContent }
```

Every IANA-registered code from 100 to 511 is a named variant — nothing more, nothing less. Full list on [docs.rs/astor](https://docs.rs/astor/latest/astor/enum.Status.html).

---

## Routing

Paths use `{name}` parameter syntax. Multiple parameters per path are supported. Each `on()` call returns `self` — registrations chain.

```rust
use astor::Method;

Router::new()
    .on(Method::Delete,  "/users/{id}",            delete_user)
    .on(Method::Get,     "/orgs/{org}/repos/{repo}", get_repo)
    .on(Method::Get,     "/users/{id}",             get_user)
    .on(Method::Options, "/users",                  options_users)
    .on(Method::Patch,   "/users/{id}",             update_user)
    .on(Method::Post,    "/users",                  create_user);
```

Retrieve parameters inside the handler with `req.param("name")`. Unmatched routes return `404 Not Found` automatically. Unknown method strings are rejected before they reach a handler.

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
        httpGet: { path: /healthz, port: 3000 }  # register this route in your app
      readinessProbe:
        httpGet: { path: /readyz, port: 3000 }   # register this route in your app
```

---

Full API reference — types, methods, and examples: **[docs.rs/astor](https://docs.rs/astor)**

---

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR. See [CHANGELOG.md](CHANGELOG.md) for release history.

---

## License

MIT

[matchit]: https://github.com/ibraheemdev/matchit
