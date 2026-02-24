//! Radix-tree request router.

use std::collections::HashMap;
use std::sync::Arc;

use matchit::Router as MatchitRouter;

use crate::handler::{BoxedHandler, Handler};

/// The application router.
///
/// Routes are registered with the method-specific builder methods. Internally
/// the router maintains one [`matchit`] radix tree per HTTP method, giving
/// **O(path-length)** lookup regardless of how many routes are registered.
///
/// The builder pattern takes ownership at each step and returns a new `Router`,
/// which lets you chain calls without needing a `mut` binding:
///
/// ```rust,no_run
/// use tsu::{Router, Request, Response};
///
/// let app = Router::new()
///     .get("/", index)
///     .get("/users/:id", get_user)
///     .post("/users", create_user)
///     .delete("/users/:id", delete_user);
///
/// async fn index(_: Request) -> &'static str { "index" }
/// async fn get_user(_: Request) -> &'static str { "user" }
/// async fn create_user(_: Request) -> &'static str { "created" }
/// async fn delete_user(_: Request) -> &'static str { "deleted" }
/// ```
pub struct Router {
    /// One matchit radix tree per HTTP method.
    ///
    /// Splitting by method keeps each tree small and avoids encoding the
    /// method into the path key. Most applications only use GET and POST.
    routes: HashMap<http::Method, MatchitRouter<BoxedHandler>>,
}

impl Router {
    /// Creates an empty router.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Registers a `GET` route.
    pub fn get(self, path: &str, handler: impl Handler) -> Self {
        self.add(http::Method::GET, path, handler)
    }

    /// Registers a `POST` route.
    pub fn post(self, path: &str, handler: impl Handler) -> Self {
        self.add(http::Method::POST, path, handler)
    }

    /// Registers a `PUT` route.
    pub fn put(self, path: &str, handler: impl Handler) -> Self {
        self.add(http::Method::PUT, path, handler)
    }

    /// Registers a `DELETE` route.
    pub fn delete(self, path: &str, handler: impl Handler) -> Self {
        self.add(http::Method::DELETE, path, handler)
    }

    /// Registers a `PATCH` route.
    pub fn patch(self, path: &str, handler: impl Handler) -> Self {
        self.add(http::Method::PATCH, path, handler)
    }

    /// Registers a route for an arbitrary HTTP method.
    pub fn route(self, method: http::Method, path: &str, handler: impl Handler) -> Self {
        self.add(method, path, handler)
    }

    /// Internal: type-erases the handler and inserts it into the right tree.
    fn add(mut self, method: http::Method, path: &str, handler: impl Handler) -> Self {
        let boxed = handler.into_boxed_handler();
        self.routes
            .entry(method)
            .or_insert_with(MatchitRouter::new)
            .insert(path, boxed)
            .unwrap_or_else(|e| panic!("invalid route `{path}`: {e}"));
        self
    }

    /// Resolves a method + path to its handler and extracted path parameters.
    ///
    /// Returns `None` if no route matches — the caller is responsible for
    /// returning a 404 response in that case.
    pub(crate) fn lookup(
        &self,
        method: &http::Method,
        path: &str,
    ) -> Option<(BoxedHandler, HashMap<String, String>)> {
        let tree = self.routes.get(method)?;
        let matched = tree.at(path).ok()?;

        // Clone the Arc — cheap atomic reference-count increment.
        let handler = Arc::clone(matched.value);

        // Collect path params into owned Strings so the handler owns them
        // without holding a reference into the matchit internals.
        let params = matched
            .params
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();

        Some((handler, params))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
