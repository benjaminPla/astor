//! Outgoing HTTP response type and the [`IntoResponse`] conversion trait.
//!
//! You should not need to think about this module directly. Build a [`Response`]
//! in your handler and return it. That is the entire job description.

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::status::Status;

// ── ContentType ───────────────────────────────────────────────────────────────

/// Common content-type values for use with [`ResponseBuilder::bytes`].
pub enum ContentType {
    Csv,          // text/csv
    EventStream,  // text/event-stream  (SSE)
    FormData,     // application/x-www-form-urlencoded
    Html,         // text/html; charset=utf-8
    Json,         // application/json
    MsgPack,      // application/msgpack
    OctetStream,  // application/octet-stream  (binary / file download)
    Pdf,          // application/pdf
    Text,         // text/plain; charset=utf-8
    Xml,          // application/xml
}

impl ContentType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Csv         => "text/csv",
            Self::EventStream => "text/event-stream",
            Self::FormData    => "application/x-www-form-urlencoded",
            Self::Html        => "text/html; charset=utf-8",
            Self::Json        => "application/json",
            Self::MsgPack     => "application/msgpack",
            Self::OctetStream => "application/octet-stream",
            Self::Pdf         => "application/pdf",
            Self::Text        => "text/plain; charset=utf-8",
            Self::Xml         => "application/xml",
        }
    }
}

// ── Response ─────────────────────────────────────────────────────────────────

/// An outgoing HTTP response.
///
/// # Shortcuts (200 OK, no custom headers needed)
///
/// ```rust
/// use tsu::{Response, Status};
///
/// Response::json(br#"{"id":1}"#.to_vec());
/// Response::text("hello");
/// Response::status(Status::NoContent);
/// ```
///
/// # Builder (custom status or headers)
///
/// ```rust
/// use tsu::{Response, ContentType, Status};
///
/// Response::builder()
///     .status(Status::Created)
///     .header("location", "/users/42")
///     .json(br#"{"id":42}"#.to_vec());
///
/// Response::builder()
///     .status(Status::Ok)
///     .bytes(ContentType::Xml, b"<ok/>".to_vec());
/// ```
pub struct Response {
    pub(crate) body: Vec<u8>,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) status: u16,
}

impl Response {
    /// `200 OK` — `application/json`.
    ///
    /// Pass bytes from your serialiser directly — no intermediate allocation:
    /// - serde_json: `serde_json::to_vec(&val).unwrap()`
    /// - hand-built: `format!(r#"{{"id":{id}}}"#).into_bytes()`  ← zero-cost
    pub fn json(body: Vec<u8>) -> Self {
        Self::bytes_raw("application/json", body)
    }

    /// `200 OK` — `text/plain; charset=utf-8`.
    pub fn text(body: impl Into<String>) -> Self {
        Self::bytes_raw("text/plain; charset=utf-8", body.into().into_bytes())
    }

    /// Response with no body.
    pub fn status(code: Status) -> Self {
        Self { body: Vec::new(), headers: Vec::new(), status: code.into() }
    }

    /// Builder for responses that need a custom status or extra headers.
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder { headers: Vec::new(), status: Status::Ok.into() }
    }

    fn bytes_raw(content_type: &str, body: Vec<u8>) -> Self {
        Self {
            body,
            headers: vec![("content-type".to_owned(), content_type.to_owned())],
            status: Status::Ok.into(),
        }
    }

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

/// Fluent builder for [`Response`].
///
/// Obtain via [`Response::builder()`]. Defaults to `Status::Ok` (200).
/// Terminated by a typed body method — you always know what you're sending.
pub struct ResponseBuilder {
    headers: Vec<(String, String)>,
    status: u16,
}

impl ResponseBuilder {
    pub fn status(mut self, code: Status) -> Self {
        self.status = code.into();
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// Terminate with a JSON body (`application/json`).
    pub fn json(self, body: Vec<u8>) -> Response {
        self.finish("application/json", body)
    }

    /// Terminate with a plain-text body (`text/plain; charset=utf-8`).
    pub fn text(self, body: impl Into<String>) -> Response {
        self.finish("text/plain; charset=utf-8", body.into().into_bytes())
    }

    /// Terminate with a typed body. Use this for XML, HTML, binary, SSE, etc.
    pub fn bytes(self, content_type: ContentType, body: Vec<u8>) -> Response {
        self.finish(content_type.as_str(), body)
    }

    /// Terminate with no body (e.g. `Status::NoContent`, `Status::MovedPermanently`).
    pub fn no_body(self) -> Response {
        Response { body: Vec::new(), headers: self.headers, status: self.status }
    }

    fn finish(self, content_type: &str, body: Vec<u8>) -> Response {
        let mut headers = vec![("content-type".to_owned(), content_type.to_owned())];
        headers.extend(self.headers);
        Response { body, headers, status: self.status }
    }
}

// ── IntoResponse ──────────────────────────────────────────────────────────────

/// Conversion into an HTTP [`Response`].
///
/// Implement on your own types to return them directly from handlers.
///
/// # Example — typed `Json<T>` wrapper with serde
///
/// ```rust,ignore
/// use tsu::{IntoResponse, Response, Status};
/// use serde::Serialize;
///
/// struct Json<T: Serialize>(T);
///
/// impl<T: Serialize> IntoResponse for Json<T> {
///     fn into_response(self) -> Response {
///         match serde_json::to_vec(&self.0) {
///             Ok(bytes) => Response::json(bytes),
///             Err(_)    => Response::status(Status::InternalServerError),
///         }
///     }
/// }
///
/// async fn get_user(_req: Request) -> Json<User> {
///     Json(User { id: 1, name: "alice".into() })
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


/// Return a [`Status`] directly from a handler: `return Status::NotFound`
impl IntoResponse for Status {
    fn into_response(self) -> Response { Response::status(self) }
}

// ── Status reason phrases ─────────────────────────────────────────────────────

fn status_reason(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Content Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a Teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Content",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _   => "",
    }
}
