//! Incoming HTTP request type.
//!
//! Parsed from the raw TCP stream by the server. By the time your handler
//! sees it, it is a clean struct. No ceremony.

use std::collections::HashMap;

use crate::method::Method;

/// An incoming HTTP request, parsed from the raw TCP stream.
pub struct Request {
    pub(crate) body: Vec<u8>,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) method: Method,
    pub(crate) params: HashMap<String, String>,
    pub(crate) path: String,
}

impl Request {
    pub(crate) fn new(
        body: Vec<u8>,
        headers: Vec<(String, String)>,
        method: Method,
        params: HashMap<String, String>,
        path: String,
    ) -> Self {
        Self { body, headers, method, params, path }
    }

    pub fn method(&self) -> Method { self.method }
    pub fn path(&self) -> &str { &self.path }
    pub fn headers(&self) -> &[(String, String)] { &self.headers }
    pub fn body(&self) -> &[u8] { &self.body }

    /// Case-insensitive header lookup.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Returns a named path parameter.
    ///
    /// For a route `/users/:id`, `req.param("id")` on `/users/42` returns `Some("42")`.
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }
}
