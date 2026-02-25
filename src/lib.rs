//! # astor
//!
//! A minimal HTTP framework for Rust services behind a reverse proxy.
//! Nothing more. Nothing less.
//!
//! ## The contract
//!
//! nginx handles TLS, rate limiting, slow clients, and body-size limits.
//! astor does not — by design. The proxy does proxy things. The framework
//! does framework things.
//!
//! What nginx / ingress already owns — astor intentionally ignores:
//!
//! - **Body-size limits** — `client_max_body_size` in nginx
//! - **Rate limiting** — `limit_req` / ingress-nginx annotations
//! - **Slow-client protection** — nginx timeout and buffer settings
//! - **TLS termination** — nginx SSL / k8s ingress
//!
//! What's left for astor:
//!
//! - Radix-tree routing — O(path-length) lookup via [`matchit`]
//! - Async I/O — tokio, raw HTTP/1.1, no hyper
//! - Graceful shutdown — SIGTERM / Ctrl-C, drains in-flight requests
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use astor::{Method, Request, Response, Router, Server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .on(Method::Get, "/",           hello)
//!         .on(Method::Get, "/users/{id}", get_user);
//!
//!     Server::bind("0.0.0.0:3000")
//!         .serve(app)
//!         .await
//!         .unwrap();
//! }
//!
//! async fn hello(_req: Request) -> Response {
//!     Response::text("Hello from astor!")
//! }
//!
//! async fn get_user(req: Request) -> Response {
//!     let id = req.param("id").unwrap_or("unknown");
//!     Response::text(format!("User: {id}"))
//! }
//! ```

mod error;
mod handler;
mod method;
mod request;
mod response;
mod router;
mod server;
mod status;

pub mod middleware;

pub use error::Error;
pub use handler::Handler;
pub use method::Method;
pub use request::Request;
pub use response::{ContentType, IntoResponse, Response};
pub use router::Router;
pub use server::Server;
pub use status::Status;
