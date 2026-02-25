//! # astor
//!
//! A minimal HTTP framework for Rust services behind a reverse proxy.
//! Nothing more. Nothing less.
//!
//! ## The contract
//!
//! nginx handles TLS, rate limiting, slow clients, and body-size limits.
//! astor does not — by design. The proxy does proxy things. The framework
//! does framework things. Every feature astor skips is one nginx already
//! ships, tested at scale, at no cost to you.
//!
//! What nginx / ingress already owns — astor intentionally ignores:
//!
//! - **Body-size limits** — `client_max_body_size` in nginx
//! - **Rate limiting** — `limit_req` / ingress-nginx annotations
//! - **Slow-client protection** — nginx timeout and buffer settings
//! - **TLS termination** — nginx SSL / k8s ingress
//!
//! What's left for astor — the only part that changes between applications:
//!
//! - Radix-tree routing — O(path-length) lookup via [`matchit`]
//! - Async I/O — tokio, raw HTTP/1.1, no hyper
//! - Graceful shutdown — SIGTERM / Ctrl-C, drains in-flight requests
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use astor::{Method, Request, Response, Router, Server, Status};
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .on(Method::Get,  "/users/{id}", get_user)
//!         .on(Method::Post, "/users",      create_user);
//!
//!     Server::bind("0.0.0.0:3000").serve(app).await.unwrap();
//! }
//!
//! async fn get_user(req: Request) -> Response {
//!     let id = req.param("id").unwrap_or("unknown");
//!     // astor sends bytes — it doesn't care how you build them:
//!     //   serde_json::to_vec(&user).unwrap()
//!     //   format!(r#"{{"id":"{id}"}}"#).into_bytes()
//!     # let bytes: Vec<u8> = vec![];
//!     Response::json(bytes)
//! }
//!
//! async fn create_user(req: Request) -> Response {
//!     if req.body().is_empty() {
//!         return Response::status(Status::BadRequest);
//!     }
//!     # let bytes: Vec<u8> = vec![];
//!     Response::builder()
//!         .status(Status::Created)
//!         .header("location", "/users/99")
//!         .json(bytes)
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
