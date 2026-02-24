//! Incoming HTTP request type.

use std::collections::HashMap;

/// An incoming HTTP request, parsed from the raw TCP stream.
pub struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
    pub(crate) params: HashMap<String, String>,
}

impl Request {
    pub(crate) fn new(
        method: String,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        params: HashMap<String, String>,
    ) -> Self {
        Self { method, path, headers, body, params }
    }

    pub fn method(&self) -> &str { &self.method }
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
