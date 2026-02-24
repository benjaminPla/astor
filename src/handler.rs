//! Handler trait and type erasure.
//!
//! # How async handlers are stored
//!
//! The router needs to hold handlers of *different* types in a single
//! `HashMap<Method, Tree>`. Rust collections can only hold one concrete type,
//! so we use **trait objects** (`dyn ErasedHandler`) to hide the concrete
//! handler type behind a common interface and store everything uniformly.
//!
//! The chain from user code to vtable call is:
//!
//! ```text
//! async fn hello(req: Request) -> Response { … }   ← user writes this
//!        ↓ router.get("/", hello)
//! hello.into_boxed_handler()                       ← Handler blanket impl
//!        ↓
//! Arc::new(FnHandler(hello))                       ← heap-allocated wrapper
//!        ↓  stored as BoxedHandler = Arc<dyn ErasedHandler>
//! handler.call(req)  at request time               ← one vtable dispatch
//!        ↓
//! Box::pin(async { hello(req).await.into_response() })  ← BoxFuture
//! ```
//!
//! The only runtime cost per request is **one Arc clone** (atomic inc) +
//! **one virtual call** — negligible compared to network I/O.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::request::Request;
use crate::response::{IntoResponse, Response};

// ── Internal types ────────────────────────────────────────────────────────────

/// A heap-allocated, type-erased future that resolves to a [`Response`].
///
/// `Pin<Box<…>>` is required because the async runtime must be able to poll
/// the future in-place — it cannot move it in memory after the first poll.
/// `Send + 'static` let tokio move the future across threads safely.
pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

/// Internal dispatch interface.
///
/// `#[doc(hidden)] pub` rather than `pub(crate)` because it appears in the
/// return type of the public `Handler` trait's `into_boxed_handler` method.
/// External crates cannot usefully interact with this trait.
#[doc(hidden)]
pub trait ErasedHandler {
    fn call(&self, req: Request) -> BoxFuture;
}

/// A heap-allocated, type-erased handler shared across concurrent requests.
///
/// `#[doc(hidden)] pub` for the same reason as `ErasedHandler`.
/// `Arc` gives us cheap, thread-safe shared ownership (one atomic reference
/// count increment per request) without copying the handler.
#[doc(hidden)]
pub type BoxedHandler = Arc<dyn ErasedHandler + Send + Sync + 'static>;

// ── Public Handler trait ──────────────────────────────────────────────────────

/// Implemented for every valid route handler.
///
/// You never implement this yourself. It is automatically satisfied for any
/// `async fn` with the signature:
///
/// ```text
/// async fn name(req: Request) -> impl IntoResponse
/// ```
///
/// The trait is **sealed** (via the private `Sealed` supertrait): only the
/// blanket impl below can satisfy it. This prevents accidental misuse and
/// keeps the API surface stable across versions.
pub trait Handler: private::Sealed + Send + Sync + 'static {
    #[doc(hidden)]
    fn into_boxed_handler(self) -> BoxedHandler;
}

/// The sealing module. Because `Sealed` is private, external crates cannot
/// name it and therefore cannot implement `Handler` on their own types.
mod private {
    pub trait Sealed {}
}

// ── Blanket implementations ───────────────────────────────────────────────────

/// Implement the sealing trait for any function with the right signature.
///
/// `Fn(Request) -> Fut` covers:
///   - named `async fn` items
///   - `async` closures (when they stabilise)
///   - any struct that implements `Fn`
impl<F, Fut, R> private::Sealed for F
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + Send + 'static,
{
}

/// Implement `Handler` for any function with the right signature.
impl<F, Fut, R> Handler for F
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + Send + 'static,
{
    fn into_boxed_handler(self) -> BoxedHandler {
        Arc::new(FnHandler(self))
    }
}

// ── Concrete wrapper ──────────────────────────────────────────────────────────

/// Newtype wrapper that holds a concrete handler `F` and implements
/// [`ErasedHandler`], bridging the typed world to the trait-object world.
struct FnHandler<F>(F);

impl<F, Fut, R> ErasedHandler for FnHandler<F>
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + Send + 'static,
{
    fn call(&self, req: Request) -> BoxFuture {
        // Call the wrapped function — this returns the concrete `Fut`.
        // We then map it to `Response` via `IntoResponse` and box the whole
        // thing so the return type matches the trait signature.
        let fut = (self.0)(req);
        Box::pin(async move { fut.await.into_response() })
    }
}
