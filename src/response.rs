//! Outgoing HTTP response type and the [`IntoResponse`] conversion trait.

use bytes::Bytes;
use tokio::io::{AsyncWrite, AsyncWriteExt};

// ── Response ─────────────────────────────────────────────────────────────────

/// An outgoing HTTP response.
///
/// ```rust
/// use tsu::Response;
///
/// let r1 = Response::text("Hello!");
/// let r2 = Response::json(r#"{"ok": true}"#);
/// let r3 = Response::status(204);
/// let r4 = Response::builder(201)
///     .header("location", "/users/42")
///     .body("created".into());
/// ```
pub struct Response {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Bytes,
}

impl Response {
    pub fn builder(status: u16) -> ResponseBuilder {
        ResponseBuilder::new(status)
    }

    /// `200 OK` with a `text/plain; charset=utf-8` body.
    pub fn text(body: impl Into<String>) -> Self {
        ResponseBuilder::new(200)
            .header("content-type", "text/plain; charset=utf-8")
            .body(body.into())
    }

    /// `200 OK` with an `application/json` body.
    pub fn json(body: impl Into<String>) -> Self {
        ResponseBuilder::new(200)
            .header("content-type", "application/json")
            .body(body.into())
    }

    /// Response with no body and the given status code.
    pub fn status(code: u16) -> Self {
        ResponseBuilder::new(code).body(String::new())
    }

    /// Serialises the response onto `writer` as HTTP/1.1.
    pub(crate) async fn write_to<W: AsyncWrite + Unpin>(
        self,
        writer: &mut W,
    ) -> std::io::Result<()> {
        writer.write_all(
            format!("HTTP/1.1 {} {}\r\n", self.status, status_reason(self.status)).as_bytes(),
        ).await?;
        writer.write_all(
            format!("content-length: {}\r\n", self.body.len()).as_bytes(),
        ).await?;
        for (name, value) in &self.headers {
            writer.write_all(format!("{name}: {value}\r\n").as_bytes()).await?;
        }
        writer.write_all(b"\r\n").await?;
        writer.write_all(&self.body).await?;
        writer.flush().await
    }
}

// ── ResponseBuilder ───────────────────────────────────────────────────────────

pub struct ResponseBuilder {
    status: u16,
    headers: Vec<(String, String)>,
}

impl ResponseBuilder {
    fn new(status: u16) -> Self {
        Self { status, headers: Vec::new() }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn body(self, body: String) -> Response {
        Response { status: self.status, headers: self.headers, body: Bytes::from(body) }
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
    fn into_response(self) -> Response { self }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response { Response::text(self) }
}

impl IntoResponse for String {
    fn into_response(self) -> Response { Response::text(self) }
}

/// Return a bare status code: `return 404u16`
impl IntoResponse for u16 {
    fn into_response(self) -> Response { Response::status(self) }
}

// ── Status reason phrases ─────────────────────────────────────────────────────

fn status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _   => "",
    }
}
