//! # tsu
//!
//! A fast, minimal HTTP framework for applications deployed behind a reverse
//! proxy (nginx, ingress-nginx on Kubernetes).
//!
//! ## Design contract
//!
//! tsu delegates to the reverse-proxy layer:
//!
//! - **TLS termination** — handled by nginx / ingress
//! - **Body-size limits** — `client_max_body_size` in nginx config
//! - **Rate limiting** — nginx `limit_req` / ingress-nginx annotations
//! - **Slow-client protection** — nginx timeout and buffer settings
//!
//! tsu owns:
//!
//! - Ultra-fast radix-tree routing (O(path-length) lookup via [`matchit`])
//! - Async request handling via tokio + hyper
//! - Graceful shutdown on SIGTERM / Ctrl-C — mandatory for Kubernetes
//! - Built-in `/healthz` and `/readyz` endpoints for k8s probes
//! - Structured tracing via the [`tracing`] crate
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use tsu::{Router, Server, Request, Response};
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .get("/", hello)
//!         .get("/users/:id", get_user);
//!
//!     Server::bind("0.0.0.0:3000")
//!         .serve(app)
//!         .await
//!         .unwrap();
//! }
//!
//! async fn hello(_req: Request) -> Response {
//!     Response::text("Hello from tsu!")
//! }
//!
//! async fn get_user(req: Request) -> Response {
//!     let id = req.param("id").unwrap_or("unknown");
//!     Response::text(format!("User: {id}"))
//! }
//! ```

mod error;
mod handler;
mod request;
mod response;
mod router;
mod server;
mod status;

pub mod health;
pub mod middleware;

pub use error::Error;
pub use handler::Handler;
pub use request::Request;
pub use response::{ContentType, IntoResponse, Response};
pub use status::Status;
pub use router::Router;
pub use server::Server;
