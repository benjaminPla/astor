//! Unified error type.

use std::fmt;

/// The error type returned by tsu's fallible operations.
///
/// Application-level errors (404, 422, etc.) are expressed as HTTP
/// [`Response`](crate::Response) values, not as `Error`s. This type surfaces
/// infrastructure failures: binding to a port, accepting a connection, or
/// malformed HTTP from an unexpected client.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

#[derive(Debug)]
enum ErrorKind {
    Io(std::io::Error),
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_ref() {
            ErrorKind::Io(e) => write!(f, "io: {e}"),
            ErrorKind::Parse(s) => write!(f, "parse: {s}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.0.as_ref() {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::Parse(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(Box::new(ErrorKind::Io(e)))
    }
}

impl Error {
    pub(crate) fn parse(msg: impl Into<String>) -> Self {
        Self(Box::new(ErrorKind::Parse(msg.into())))
    }
}
