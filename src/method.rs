//! HTTP method as a typed enum.
//!
//! Covers RFC 9110 standard methods, WebDAV extensions (RFC 4918 / 4791 / 3253 / 5323),
//! and `PURGE` used by nginx and Varnish for cache invalidation.
//!
//! Unknown method strings are rejected at the server level with `405 Method Not Allowed`
//! before they ever reach a handler.

use std::fmt;
use std::str::FromStr;

/// A known HTTP method.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Method {
    // RFC 9110 ─────────────────────────────────────────────────────────────────
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
    Trace,
    // WebDAV RFC 4918 ──────────────────────────────────────────────────────────
    Copy,
    Lock,
    Mkcol,
    Move,
    Propfind,
    Proppatch,
    Unlock,
    // WebDAV extensions ────────────────────────────────────────────────────────
    Mkcalendar, // RFC 4791 — CalDAV
    Report,     // RFC 3253
    Search,     // RFC 5323
    // Cache invalidation ───────────────────────────────────────────────────────
    Purge, // nginx / Varnish
}

impl Method {
    /// Returns the uppercase wire representation (e.g. `"GET"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connect    => "CONNECT",
            Self::Copy       => "COPY",
            Self::Delete     => "DELETE",
            Self::Get        => "GET",
            Self::Head       => "HEAD",
            Self::Lock       => "LOCK",
            Self::Mkcalendar => "MKCALENDAR",
            Self::Mkcol      => "MKCOL",
            Self::Move       => "MOVE",
            Self::Options    => "OPTIONS",
            Self::Patch      => "PATCH",
            Self::Post       => "POST",
            Self::Propfind   => "PROPFIND",
            Self::Proppatch  => "PROPPATCH",
            Self::Purge      => "PURGE",
            Self::Put        => "PUT",
            Self::Report     => "REPORT",
            Self::Search     => "SEARCH",
            Self::Trace      => "TRACE",
            Self::Unlock     => "UNLOCK",
        }
    }
}

/// Parses an uppercase method string (e.g. `"GET"`). Case-sensitive per RFC 9110 §9.1.
impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CONNECT"    => Ok(Self::Connect),
            "COPY"       => Ok(Self::Copy),
            "DELETE"     => Ok(Self::Delete),
            "GET"        => Ok(Self::Get),
            "HEAD"       => Ok(Self::Head),
            "LOCK"       => Ok(Self::Lock),
            "MKCALENDAR" => Ok(Self::Mkcalendar),
            "MKCOL"      => Ok(Self::Mkcol),
            "MOVE"       => Ok(Self::Move),
            "OPTIONS"    => Ok(Self::Options),
            "PATCH"      => Ok(Self::Patch),
            "POST"       => Ok(Self::Post),
            "PROPFIND"   => Ok(Self::Propfind),
            "PROPPATCH"  => Ok(Self::Proppatch),
            "PURGE"      => Ok(Self::Purge),
            "PUT"        => Ok(Self::Put),
            "REPORT"     => Ok(Self::Report),
            "SEARCH"     => Ok(Self::Search),
            "TRACE"      => Ok(Self::Trace),
            "UNLOCK"     => Ok(Self::Unlock),
            _            => Err(()),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
