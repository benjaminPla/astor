//! Incoming HTTP request type.
//!
//! Parsed from the raw TCP stream by the server. By the time your handler
//! receives a [`Request`], the proxy has already validated and buffered the
//! input. You get a clean struct — no ceremony, no streaming.
//!
//! # Accessing request data
//!
//! ```rust,no_run
//! use astor::{Request, Response, Status};
//!
//! async fn handler(req: Request) -> Response {
//!     // Path parameter — registered as {id} in the route
//!     let id = req.param("id").unwrap_or("unknown");
//!
//!     // Query parameter — GET /users/42?verbose=true
//!     let verbose = req.query("verbose").unwrap_or("false");
//!
//!     // Single header — case-insensitive
//!     let auth = req.header("authorization");
//!
//!     // Raw body bytes — parse with whatever you want
//!     if req.body().is_empty() {
//!         return Response::status(Status::BadRequest);
//!     }
//!
//!     Response::text(format!("{id} verbose={verbose}"))
//! }
//! ```

use std::collections::HashMap;

use crate::method::Method;

/// An incoming HTTP request, parsed from the raw TCP stream.
///
/// Constructed by the server before dispatch. Fields are read-only — handlers
/// receive the request and return a [`Response`][crate::Response].
pub struct Request {
    pub(crate) body: Vec<u8>,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) method: Method,
    pub(crate) params: HashMap<String, String>,
    pub(crate) path: String,
    pub(crate) query: HashMap<String, String>,
    pub(crate) raw_query: String,
}

impl Request {
    pub(crate) fn new(
        body: Vec<u8>,
        headers: Vec<(String, String)>,
        method: Method,
        params: HashMap<String, String>,
        path: String,
        raw_query: String,
    ) -> Self {
        let query = parse_query(&raw_query);
        Self { body, headers, method, params, path, query, raw_query }
    }

    /// Returns the HTTP method.
    pub fn method(&self) -> Method { self.method }

    /// Returns the request path, without the query string.
    ///
    /// For a request URI of `/users/42?page=1` this returns `/users/42`.
    pub fn path(&self) -> &str { &self.path }

    /// Looks up a single query parameter by name.
    ///
    /// Returns `None` if the key is absent. For duplicate keys (e.g.
    /// `?tag=a&tag=b`) the last value wins — this is uncommon in REST APIs
    /// and consistent with most framework behaviour.
    ///
    /// Unknown parameters from external services or tracing agents are kept
    /// as-is; the handler simply ignores what it does not need.
    ///
    /// ```rust,no_run
    /// # use astor::{Request, Response, Status};
    /// async fn handler(req: Request) -> Response {
    ///     // GET /search?q=rust&page=2
    ///     let q    = req.query("q").unwrap_or("*");
    ///     let page = req.query("page").unwrap_or("1");
    ///     Response::text(format!("q={q} page={page}"))
    /// }
    /// ```
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    /// Returns the raw query string, without the leading `?`.
    ///
    /// Empty string if the request had no query string. Use this when you
    /// need the original bytes — e.g. HMAC signature verification against
    /// an external API that signs the raw query string.
    ///
    /// ```rust,no_run
    /// # use astor::{Request, Response};
    /// async fn handler(req: Request) -> Response {
    ///     // e.g. GET /hook?ts=1234&sig=abc
    ///     let raw = req.raw_query(); // "ts=1234&sig=abc"
    ///     Response::text(raw)
    /// }
    /// ```
    pub fn raw_query(&self) -> &str { &self.raw_query }

    /// Returns all request headers as name-value pairs.
    ///
    /// Header names are lowercased by nginx before reaching astor.
    /// For single-header lookup, prefer [`header`][Request::header].
    pub fn headers(&self) -> &[(String, String)] { &self.headers }

    /// Returns the raw request body as bytes.
    ///
    /// astor never interprets the bytes — parse them with whatever fits your
    /// use case:
    /// - `serde_json::from_slice(req.body())`
    /// - `simd_json::from_slice(req.body())`
    /// - hand-rolled parsing for simple formats
    ///
    /// An empty body returns an empty slice. Check `is_empty()` before
    /// attempting to parse.
    ///
    /// Body size is constrained by `client_max_body_size` in your nginx config,
    /// not by astor. Gate on `body.len()` inside the handler if you need
    /// per-route limits tighter than the global nginx setting.
    pub fn body(&self) -> &[u8] { &self.body }

    /// Case-insensitive lookup for a single header by name.
    ///
    /// Returns `None` if the header is absent. Header names are lowercased by
    /// nginx, so `"Authorization"` and `"authorization"` both match.
    ///
    /// ```rust,no_run
    /// # use astor::{Request, Response, Status};
    /// async fn handler(req: Request) -> Response {
    ///     match req.header("authorization") {
    ///         Some(token) => Response::text(token),
    ///         None        => Response::status(Status::Unauthorized),
    ///     }
    /// }
    /// ```
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Returns a named path parameter extracted by the router.
    ///
    /// For a route `/users/{id}`, `req.param("id")` on `/users/42` returns
    /// `Some("42")`. Returns `None` if the key is not in the route pattern.
    ///
    /// ```rust,no_run
    /// # use astor::{Request, Response};
    /// // Route: /orgs/{org}/repos/{repo}
    /// async fn get_repo(req: Request) -> Response {
    ///     let org  = req.param("org").unwrap();
    ///     let repo = req.param("repo").unwrap();
    ///     Response::text(format!("{org}/{repo}"))
    /// }
    /// ```
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }
}

// ── Internal ──────────────────────────────────────────────────────────────────

/// Parses `key=value&key2=value2` into a map.
///
/// - Pairs with no `=` are stored with an empty-string value.
/// - Duplicate keys: last value wins.
/// - No percent-decoding: nginx passes query strings to upstream as-is,
///   so the raw bytes are already what the client sent.
fn parse_query(raw: &str) -> HashMap<String, String> {
    if raw.is_empty() {
        return HashMap::new();
    }
    raw.split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            if k.is_empty() { return None; }
            Some((k.to_owned(), v.to_owned()))
        })
        .collect()
}
