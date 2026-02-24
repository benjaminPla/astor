//! Minimal tsu example — two routes and the built-in health checks.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example basic
//! ```
//!
//! Then try:
//!
//! ```sh
//! curl http://localhost:3000/
//! curl http://localhost:3000/users/42
//! curl http://localhost:3000/healthz
//! curl http://localhost:3000/readyz
//! curl http://localhost:3000/missing   # → 404
//! ```

use tsu::{Request, Response, Router, Server, health};

#[tokio::main]
async fn main() {
    // `fmt::init()` reads the RUST_LOG env var and prints spans/events to
    // stderr. Try: RUST_LOG=info cargo run --example basic
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // Application routes
        .get("/", hello)
        .get("/users/:id", get_user)
        // Kubernetes probes — always register these
        .get("/healthz", health::liveness)
        .get("/readyz", health::readiness);

    Server::bind("0.0.0.0:3000")
        .serve(app)
        .await
        .expect("server error");
}

async fn hello(_req: Request) -> Response {
    Response::text("Hello from tsu!")
}

async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    // Hand-written JSON — serde integration will be added in a later iteration.
    Response::json(format!(r#"{{"id": "{id}"}}"#))
}
