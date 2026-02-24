//! Incoming HTTP request type.

use std::collections::HashMap;

use http::{HeaderMap, Method, Uri, Version};

/// An incoming HTTP request.
///
/// Wraps the raw Hyper request and attaches path parameters extracted by the
/// router. Because tsu sits behind a reverse proxy, header validation and
/// body-size enforcement have already happened upstream — this type exposes
/// the data as-is.
pub struct Request {
    pub(crate) inner: http::Request<hyper::body::Incoming>,
    pub(crate) params: HashMap<String, String>,
}

impl Request {
    /// Constructs a `Request` from the raw Hyper request and router-extracted
    /// path parameters.
    ///
    /// Called internally by the server — you never construct `Request` directly
    /// in handler code.
    pub(crate) fn new(
        inner: http::Request<hyper::body::Incoming>,
        params: HashMap<String, String>,
    ) -> Self {
        Self { inner, params }
    }

    /// Returns the HTTP method (GET, POST, …).
    pub fn method(&self) -> &Method {
        self.inner.method()
    }

    /// Returns the request URI.
    pub fn uri(&self) -> &Uri {
        self.inner.uri()
    }

    /// Returns the HTTP version.
    pub fn version(&self) -> Version {
        self.inner.version()
    }

    /// Returns the request headers.
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Returns a named path parameter.
    ///
    /// For a route registered as `/users/:id`, calling `req.param("id")` on a
    /// request to `/users/42` returns `Some("42")`.
    ///
    /// Returns `None` if the route has no parameter with that name.
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }
}
