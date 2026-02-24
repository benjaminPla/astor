//! Outgoing HTTP response type and the [`IntoResponse`] conversion trait.

use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use http_body_util::Full;

// ── Response ─────────────────────────────────────────────────────────────────

/// An outgoing HTTP response.
///
/// Use the static helpers for the common cases, or [`Response::builder`] when
/// you need custom headers or a non-200 status:
///
/// ```rust
/// use tsu::Response;
/// use http::StatusCode;
///
/// let r1 = Response::text("Hello!");
/// let r2 = Response::json(r#"{"ok": true}"#);
/// let r3 = Response::status(StatusCode::NO_CONTENT);
/// let r4 = Response::builder(StatusCode::CREATED)
///     .header(http::header::LOCATION, "/users/42")
///     .body("created".into());
/// ```
pub struct Response {
    pub(crate) inner: http::Response<Full<Bytes>>,
}

impl Response {
    /// Returns a fluent builder starting from the given status code.
    pub fn builder(status: StatusCode) -> ResponseBuilder {
        ResponseBuilder::new(status)
    }

    /// `200 OK` with a `text/plain; charset=utf-8` body.
    pub fn text(body: impl Into<String>) -> Self {
        ResponseBuilder::new(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(body.into())
    }

    /// `200 OK` with an `application/json` body.
    ///
    /// The caller is responsible for serialising to a valid JSON string.
    pub fn json(body: impl Into<String>) -> Self {
        ResponseBuilder::new(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.into())
    }

    /// Response with no body and the given status code.
    pub fn status(code: StatusCode) -> Self {
        ResponseBuilder::new(code).body(String::new())
    }

    /// Unwraps into the inner Hyper response for the connection layer.
    ///
    /// This is `pub(crate)` — user code never needs to call it.
    pub(crate) fn into_inner(self) -> http::Response<Full<Bytes>> {
        self.inner
    }
}

// ── ResponseBuilder ───────────────────────────────────────────────────────────

/// Fluent builder for [`Response`].
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
}

impl ResponseBuilder {
    fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
        }
    }

    /// Appends a response header.
    pub fn header(mut self, name: HeaderName, value: &'static str) -> Self {
        self.headers.insert(name, HeaderValue::from_static(value));
        self
    }

    /// Finalises the response with the given body string.
    pub fn body(self, body: String) -> Response {
        let bytes = Bytes::from(body);
        let mut res = http::Response::new(Full::new(bytes));
        *res.status_mut() = self.status;
        *res.headers_mut() = self.headers;
        Response { inner: res }
    }
}

// ── IntoResponse ──────────────────────────────────────────────────────────────

/// Conversion into an HTTP [`Response`].
///
/// Implement this on your own types to return them directly from handlers:
///
/// ```rust
/// use tsu::{IntoResponse, Response};
///
/// struct Json(String);
///
/// impl IntoResponse for Json {
///     fn into_response(self) -> Response {
///         Response::json(self.0)
///     }
/// }
/// ```
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::text(self)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::text(self)
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::status(self)
    }
}
