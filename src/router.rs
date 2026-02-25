//! Radix-tree request router.
//!
//! One tree per HTTP method. O(path-length) lookup. No magic, no middleware
//! stack, no reflection. You register a path, you get a handler. That is all.

use std::collections::HashMap;
use std::sync::Arc;

use matchit::Router as MatchitRouter;

use crate::handler::{BoxedHandler, Handler};

/// The application router.
///
/// One [`matchit`] radix tree per HTTP method — O(path-length) lookup.
/// Builder pattern: each registration takes ownership and returns a new `Router`.
pub struct Router {
    routes: HashMap<String, MatchitRouter<BoxedHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    pub fn delete(self, path: &str, handler: impl Handler) -> Self {
        self.add("DELETE", path, handler)
    }

    pub fn get(self, path: &str, handler: impl Handler) -> Self {
        self.add("GET", path, handler)
    }

    pub fn patch(self, path: &str, handler: impl Handler) -> Self {
        self.add("PATCH", path, handler)
    }

    pub fn post(self, path: &str, handler: impl Handler) -> Self {
        self.add("POST", path, handler)
    }

    pub fn put(self, path: &str, handler: impl Handler) -> Self {
        self.add("PUT", path, handler)
    }

    /// Registers a route for an arbitrary HTTP method string (e.g. `"OPTIONS"`).
    pub fn route(self, method: &str, path: &str, handler: impl Handler) -> Self {
        self.add(method, path, handler)
    }

    fn add(mut self, method: &str, path: &str, handler: impl Handler) -> Self {
        self.routes
            .entry(method.to_uppercase())
            .or_insert_with(MatchitRouter::new)
            .insert(path, handler.into_boxed_handler())
            .unwrap_or_else(|e| panic!("invalid route `{path}`: {e}"));
        self
    }

    pub(crate) fn lookup(
        &self,
        method: &str,
        path: &str,
    ) -> Option<(BoxedHandler, HashMap<String, String>)> {
        let tree = self.routes.get(method)?;
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
