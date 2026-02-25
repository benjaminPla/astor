//! Built-in Kubernetes health-check handlers.
//!
//! Kubernetes asks two questions. astor answers them.
//!
//! | Probe | Path | Question |
//! |---|---|---|
//! | **Liveness** | `/healthz` | Is the process alive? Failure → restart. |
//! | **Readiness** | `/readyz` | Can the pod serve traffic? Failure → pulled from load-balancer. |
//!
//! Register them on your router:
//!
//! ```rust,no_run
//! use astor::{Router, health};
//!
//! let app = Router::new()
//!     .get("/healthz", health::liveness)
//!     .get("/readyz", health::readiness);
//! ```
//!
//! Override `readiness` with a custom handler if you need to gate on
//! dependency availability (database connections, downstream services, etc.):
//!
//! ```rust,no_run
//! use astor::{Request, Response, Status};
//!
//! async fn readiness(_req: Request) -> Response {
//!     if dependencies_are_healthy().await {
//!         Response::text("ready")
//!     } else {
//!         Response::status(Status::ServiceUnavailable)
//!     }
//! }
//!
//! async fn dependencies_are_healthy() -> bool { true }
//! ```

use crate::{Request, Response};

/// Kubernetes liveness probe handler.
///
/// Always returns `200 OK` with body `"ok"`. If the process can respond to
/// HTTP at all, it is alive — this handler intentionally has no dependencies.
pub async fn liveness(_req: Request) -> Response {
    Response::text("ok")
}

/// Kubernetes readiness probe handler (default implementation).
///
/// Returns `200 OK` with body `"ready"`. Replace this with your own handler
/// if your application needs a warm-up period or must verify dependency health
/// before accepting traffic.
pub async fn readiness(_req: Request) -> Response {
    Response::text("ready")
}
