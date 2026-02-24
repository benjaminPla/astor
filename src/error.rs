//! Unified error type.

use std::fmt;

/// The error type returned by tsu's fallible operations.
///
/// Application-level errors (404, 422, etc.) are expressed as HTTP
/// [`Response`](crate::Response) values, not as `Error`s. This type surfaces
/// infrastructure failures: binding to a port or accepting a connection.
#[derive(Debug)]
pub struct Error(std::io::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "io: {}", self.0)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e)
    }
}
