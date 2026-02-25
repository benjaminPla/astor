//! Radix-tree request router.
//!
//! One tree per HTTP method. O(path-length) lookup. No magic, no middleware
//! stack, no reflection. You register a path, you get a handler. That is all.

use std::collections::HashMap;
use std::sync::Arc;

use matchit::Router as MatchitRouter;

use crate::handler::{BoxedHandler, Handler};
use crate::method::Method;

/// The application router.
///
/// One radix tree per HTTP method — O(path-length) lookup, no allocations on
/// the hot path. Build it once at startup; pass it to [`Server::serve`].
/// Each [`Router::on`] call returns `self` so registrations chain naturally.
pub struct Router {
    routes: HashMap<Method, MatchitRouter<BoxedHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    /// Register a handler for a method + path pair. Returns `self` for chaining.
    ///
    /// Path parameters use `{name}` syntax — `req.param("name")` retrieves them:
    ///
    /// ```rust,no_run
    /// # use astor::{Method, Request, Response, Router};
    /// # async fn get_user(_: Request) -> Response { Response::text("") }
    /// # async fn create_user(_: Request) -> Response { Response::text("") }
    /// # async fn delete_user(_: Request) -> Response { Response::text("") }
    /// Router::new()
    ///     .on(Method::Delete, "/users/{id}", delete_user)
    ///     .on(Method::Get,    "/users/{id}", get_user)
    ///     .on(Method::Post,   "/users",      create_user);
    /// ```
    pub fn on(self, method: Method, path: &str, handler: impl Handler) -> Self {
        self.add(method, path, handler)
    }

    fn add(mut self, method: Method, path: &str, handler: impl Handler) -> Self {
        self.routes
            .entry(method)
            .or_default()
            .insert(path, handler.into_boxed_handler())
            .unwrap_or_else(|e| panic!("invalid route `{path}`: {e}"));
        self
    }

    pub(crate) fn lookup(
        &self,
        method: Method,
        path: &str,
    ) -> Option<(BoxedHandler, HashMap<String, String>)> {
        let tree = self.routes.get(&method)?;
        let matched = tree.at(path).ok()?;
        let handler = Arc::clone(matched.value);
        let params = matched.params.iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Some((handler, params))
    }
}

impl Default for Router {
    fn default() -> Self { Self::new() }
}
