//! HTTP status codes as a typed enum.
//!
//! Use [`Status`] anywhere a status code is accepted — `Response::status()`,
//! `Response::builder().status()`, or as a bare handler return value.
//!
//! ```rust
//! use astor::{Response, Status};
//!
//! // status-only, no body
//! Response::status(Status::NoContent);
//!
//! // bytes — astor doesn't care: serde_json::to_vec(&val).unwrap(), format!(r#"..."#).into_bytes(), etc.
//! # let bytes: Vec<u8> = vec![];
//! Response::builder()
//!     .status(Status::Created)
//!     .header("location", "/users/42")
//!     .json(bytes);
//!
//! // return Status directly from a handler — astor wraps it
//! async fn delete_user(_req: astor::Request) -> Status {
//!     Status::NoContent
//! }
//! ```

/// All IANA-registered HTTP status codes.
#[allow(clippy::enum_variant_names)]
pub enum Status {
    // ── 1xx Informational ─────────────────────────────────────────────────────
    Continue,                      // 100
    SwitchingProtocols,            // 101
    Processing,                    // 102
    EarlyHints,                    // 103

    // ── 2xx Success ───────────────────────────────────────────────────────────
    Ok,                            // 200
    Created,                       // 201
    Accepted,                      // 202
    NonAuthoritativeInformation,   // 203
    NoContent,                     // 204
    ResetContent,                  // 205
    PartialContent,                // 206
    MultiStatus,                   // 207
    AlreadyReported,               // 208
    ImUsed,                        // 226

    // ── 3xx Redirection ───────────────────────────────────────────────────────
    MultipleChoices,               // 300
    MovedPermanently,              // 301
    Found,                         // 302
    SeeOther,                      // 303
    NotModified,                   // 304
    TemporaryRedirect,             // 307
    PermanentRedirect,             // 308

    // ── 4xx Client errors ─────────────────────────────────────────────────────
    BadRequest,                    // 400
    Unauthorized,                  // 401
    PaymentRequired,               // 402
    Forbidden,                     // 403
    NotFound,                      // 404
    MethodNotAllowed,              // 405
    NotAcceptable,                 // 406
    ProxyAuthenticationRequired,   // 407
    RequestTimeout,                // 408
    Conflict,                      // 409
    Gone,                          // 410
    LengthRequired,                // 411
    PreconditionFailed,            // 412
    ContentTooLarge,               // 413
    UriTooLong,                    // 414
    UnsupportedMediaType,          // 415
    RangeNotSatisfiable,           // 416
    ExpectationFailed,             // 417
    ImATeapot,                     // 418
    MisdirectedRequest,            // 421
    UnprocessableContent,          // 422
    Locked,                        // 423
    FailedDependency,              // 424
    TooEarly,                      // 425
    UpgradeRequired,               // 426
    PreconditionRequired,          // 428
    TooManyRequests,               // 429
    RequestHeaderFieldsTooLarge,   // 431
    UnavailableForLegalReasons,    // 451

    // ── 5xx Server errors ─────────────────────────────────────────────────────
    InternalServerError,           // 500
    NotImplemented,                // 501
    BadGateway,                    // 502
    ServiceUnavailable,            // 503
    GatewayTimeout,                // 504
    HttpVersionNotSupported,       // 505
    VariantAlsoNegotiates,         // 506
    InsufficientStorage,           // 507
    LoopDetected,                  // 508
    NotExtended,                   // 510
    NetworkAuthenticationRequired, // 511
}

impl From<Status> for u16 {
    fn from(s: Status) -> u16 {
        match s {
            Status::Continue                      => 100,
            Status::SwitchingProtocols            => 101,
            Status::Processing                    => 102,
            Status::EarlyHints                    => 103,
            Status::Ok                            => 200,
            Status::Created                       => 201,
            Status::Accepted                      => 202,
            Status::NonAuthoritativeInformation   => 203,
            Status::NoContent                     => 204,
            Status::ResetContent                  => 205,
            Status::PartialContent                => 206,
            Status::MultiStatus                   => 207,
            Status::AlreadyReported               => 208,
            Status::ImUsed                        => 226,
            Status::MultipleChoices               => 300,
            Status::MovedPermanently              => 301,
            Status::Found                         => 302,
            Status::SeeOther                      => 303,
            Status::NotModified                   => 304,
            Status::TemporaryRedirect             => 307,
            Status::PermanentRedirect             => 308,
            Status::BadRequest                    => 400,
            Status::Unauthorized                  => 401,
            Status::PaymentRequired               => 402,
            Status::Forbidden                     => 403,
            Status::NotFound                      => 404,
            Status::MethodNotAllowed              => 405,
            Status::NotAcceptable                 => 406,
            Status::ProxyAuthenticationRequired   => 407,
            Status::RequestTimeout                => 408,
            Status::Conflict                      => 409,
            Status::Gone                          => 410,
            Status::LengthRequired                => 411,
            Status::PreconditionFailed            => 412,
            Status::ContentTooLarge               => 413,
            Status::UriTooLong                    => 414,
            Status::UnsupportedMediaType          => 415,
            Status::RangeNotSatisfiable           => 416,
            Status::ExpectationFailed             => 417,
            Status::ImATeapot                     => 418,
            Status::MisdirectedRequest            => 421,
            Status::UnprocessableContent          => 422,
            Status::Locked                        => 423,
            Status::FailedDependency              => 424,
            Status::TooEarly                      => 425,
            Status::UpgradeRequired               => 426,
            Status::PreconditionRequired          => 428,
            Status::TooManyRequests               => 429,
            Status::RequestHeaderFieldsTooLarge   => 431,
            Status::UnavailableForLegalReasons    => 451,
            Status::InternalServerError           => 500,
            Status::NotImplemented                => 501,
            Status::BadGateway                    => 502,
            Status::ServiceUnavailable            => 503,
            Status::GatewayTimeout                => 504,
            Status::HttpVersionNotSupported       => 505,
            Status::VariantAlsoNegotiates         => 506,
            Status::InsufficientStorage           => 507,
            Status::LoopDetected                  => 508,
            Status::NotExtended                   => 510,
            Status::NetworkAuthenticationRequired => 511,
        }
    }
}
