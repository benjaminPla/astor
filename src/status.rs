//! HTTP status codes as a typed enum.
//!
//! Every IANA-registered status code is a named [`Status`] variant. You cannot
//! construct a response with a raw integer — the type system prevents it.
//! `Status::Created` is always 201. There is no way to typo `201` as `211`,
//! or to pass `2040` when you meant `204`.
//!
//! Use [`Status`] anywhere a status code is accepted:
//! - [`Response::status`][crate::Response::status] — body-less response
//! - [`ResponseBuilder::status`][crate::response::ResponseBuilder::status] — via the builder
//! - As a bare handler return value — astor wraps it automatically
//!
//! ```rust
//! use astor::{Request, Response, Status};
//!
//! // body-less responses
//! Response::status(Status::NoContent);
//! Response::status(Status::NotFound);
//!
//! # let bytes: Vec<u8> = vec![];
//! // builder: explicit status + headers
//! Response::builder()
//!     .status(Status::Created)
//!     .header("location", "/users/42")
//!     .json(bytes);
//!
//! // return Status directly from a handler — astor wraps it
//! async fn delete_user(_req: Request) -> Status {
//!     Status::NoContent
//! }
//! ```

/// All IANA-registered HTTP status codes.
///
/// Variants are grouped by class and listed alphabetically within each group.
/// Use the variant name — never a raw integer.
#[allow(clippy::enum_variant_names)]
pub enum Status {
    // ── 1xx Informational ─────────────────────────────────────────────────────
    Continue,                      // 100
    EarlyHints,                    // 103
    Processing,                    // 102
    SwitchingProtocols,            // 101

    // ── 2xx Success ───────────────────────────────────────────────────────────
    Accepted,                      // 202
    AlreadyReported,               // 208
    Created,                       // 201
    ImUsed,                        // 226
    MultiStatus,                   // 207
    NoContent,                     // 204
    NonAuthoritativeInformation,   // 203
    Ok,                            // 200
    PartialContent,                // 206
    ResetContent,                  // 205

    // ── 3xx Redirection ───────────────────────────────────────────────────────
    Found,                         // 302
    MovedPermanently,              // 301
    MultipleChoices,               // 300
    NotModified,                   // 304
    PermanentRedirect,             // 308
    SeeOther,                      // 303
    TemporaryRedirect,             // 307

    // ── 4xx Client errors ─────────────────────────────────────────────────────
    BadRequest,                    // 400
    Conflict,                      // 409
    ContentTooLarge,               // 413
    ExpectationFailed,             // 417
    FailedDependency,              // 424
    Forbidden,                     // 403
    Gone,                          // 410
    ImATeapot,                     // 418
    LengthRequired,                // 411
    Locked,                        // 423
    MethodNotAllowed,              // 405
    MisdirectedRequest,            // 421
    NotAcceptable,                 // 406
    NotFound,                      // 404
    PaymentRequired,               // 402
    PreconditionFailed,            // 412
    PreconditionRequired,          // 428
    ProxyAuthenticationRequired,   // 407
    RangeNotSatisfiable,           // 416
    RequestHeaderFieldsTooLarge,   // 431
    RequestTimeout,                // 408
    TooEarly,                      // 425
    TooManyRequests,               // 429
    Unauthorized,                  // 401
    UnavailableForLegalReasons,    // 451
    UnprocessableContent,          // 422
    UnsupportedMediaType,          // 415
    UpgradeRequired,               // 426
    UriTooLong,                    // 414

    // ── 5xx Server errors ─────────────────────────────────────────────────────
    BadGateway,                    // 502
    GatewayTimeout,                // 504
    HttpVersionNotSupported,       // 505
    InsufficientStorage,           // 507
    InternalServerError,           // 500
    LoopDetected,                  // 508
    NetworkAuthenticationRequired, // 511
    NotExtended,                   // 510
    NotImplemented,                // 501
    ServiceUnavailable,            // 503
    VariantAlsoNegotiates,         // 506
}

impl From<Status> for u16 {
    fn from(s: Status) -> u16 {
        match s {
            Status::Accepted                      => 202,
            Status::AlreadyReported               => 208,
            Status::BadGateway                    => 502,
            Status::BadRequest                    => 400,
            Status::Conflict                      => 409,
            Status::ContentTooLarge               => 413,
            Status::Continue                      => 100,
            Status::Created                       => 201,
            Status::EarlyHints                    => 103,
            Status::ExpectationFailed             => 417,
            Status::FailedDependency              => 424,
            Status::Forbidden                     => 403,
            Status::Found                         => 302,
            Status::GatewayTimeout                => 504,
            Status::Gone                          => 410,
            Status::HttpVersionNotSupported       => 505,
            Status::ImATeapot                     => 418,
            Status::ImUsed                        => 226,
            Status::InsufficientStorage           => 507,
            Status::InternalServerError           => 500,
            Status::LengthRequired                => 411,
            Status::Locked                        => 423,
            Status::LoopDetected                  => 508,
            Status::MethodNotAllowed              => 405,
            Status::MisdirectedRequest            => 421,
            Status::MovedPermanently              => 301,
            Status::MultiStatus                   => 207,
            Status::MultipleChoices               => 300,
            Status::NetworkAuthenticationRequired => 511,
            Status::NoContent                     => 204,
            Status::NonAuthoritativeInformation   => 203,
            Status::NotAcceptable                 => 406,
            Status::NotExtended                   => 510,
            Status::NotFound                      => 404,
            Status::NotImplemented                => 501,
            Status::NotModified                   => 304,
            Status::Ok                            => 200,
            Status::PartialContent                => 206,
            Status::PaymentRequired               => 402,
            Status::PermanentRedirect             => 308,
            Status::PreconditionFailed            => 412,
            Status::PreconditionRequired          => 428,
            Status::Processing                    => 102,
            Status::ProxyAuthenticationRequired   => 407,
            Status::RangeNotSatisfiable           => 416,
            Status::RequestHeaderFieldsTooLarge   => 431,
            Status::RequestTimeout                => 408,
            Status::ResetContent                  => 205,
            Status::SeeOther                      => 303,
            Status::ServiceUnavailable            => 503,
            Status::SwitchingProtocols            => 101,
            Status::TemporaryRedirect             => 307,
            Status::TooEarly                      => 425,
            Status::TooManyRequests               => 429,
            Status::Unauthorized                  => 401,
            Status::UnavailableForLegalReasons    => 451,
            Status::UnprocessableContent          => 422,
            Status::UnsupportedMediaType          => 415,
            Status::UpgradeRequired               => 426,
            Status::UriTooLong                    => 414,
            Status::VariantAlsoNegotiates         => 506,
        }
    }
}
