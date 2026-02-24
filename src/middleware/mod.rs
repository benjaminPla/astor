//! Middleware layer.
//!
//! Middleware intercepts requests and responses and is the right place for
//! cross-cutting concerns: structured tracing, metrics, request-id injection,
//! and authentication-header inspection.
//!
//! This module is currently a placeholder. The middleware API will be
//! designed and stabilised in a subsequent iteration once the core routing
//! is solid and battle-tested.
//!
//! Planned built-in middleware:
//! - `middleware::trace` — per-request span with method, path, status, latency
