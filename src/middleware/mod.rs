//! Middleware layer — intercept requests and responses for cross-cutting concerns.
//!
//! The right place for things that apply across every handler: metrics,
//! request-id injection, authentication-header inspection, and similar.
//!
//! This module is a placeholder. The middleware API will be designed and
//! stabilised once the core routing is solid and battle-tested.
//!
//! # Planned
//!
//! - `middleware::metrics` — per-request counters and latency histograms
