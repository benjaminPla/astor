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
/// One [`matchit`] radix tree per HTTP method — O(path-length) lookup.
/// Builder pattern: each registration takes ownership and returns a new `Router`.
pub struct Router {
    routes: HashMap<Method, MatchitRouter<BoxedHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    pub fn delete(self, path: &str, handler: impl Handler) -> Self {
        self.add(Method::Delete, path, handler)
    }

    pub fn get(self, path: &str, handler: impl Handler) -> Self {
        self.add(Method::Get, path, handler)
    }

    pub fn patch(self, path: &str, handler: impl Handler) -> Self {
        self.add(Method::Patch, path, handler)
    }

    pub fn post(self, path: &str, handler: impl Handler) -> Self {
        self.add(Method::Post, path, handler)
    }

    pub fn put(self, path: &str, handler: impl Handler) -> Self {
        self.add(Method::Put, path, handler)
    }

    /// Registers a route for any known [`Method`].
    pub fn route(self, method: Method, path: &str, handler: impl Handler) -> Self {
        self.add(method, path, handler)
    }

    fn add(mut self, method: Method, path: &str, handler: impl Handler) -> Self {
        // matchit 0.8 uses `{param}` syntax; translate the conventional `:param` form.
        let path = colon_to_braces(path);
        self.routes
            .entry(method)
            .or_default()
            .insert(path.clone(), handler.into_boxed_handler())
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

// Translates `:param` segments to `{param}` for matchit 0.8+.
fn colon_to_braces(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for segment in path.split('/') {
        out.push('/');
        if let Some(name) = segment.strip_prefix(':') {
            out.push('{');
            out.push_str(name);
            out.push('}');
        } else {
            out.push_str(segment);
        }
    }
    // split('/') on "/users/:id" yields ["", "users", ":id"] — the leading
    // '/' is already added by the first iteration, strip the extra one.
    out[1..].to_owned()
}
