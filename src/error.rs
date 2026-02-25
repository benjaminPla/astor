//! Unified error type.
//!
//! Application-level problems (wrong input, missing resource) belong in the
//! response — use `Status::BadRequest`, `Status::NotFound`, etc. This type
//! surfaces infrastructure failures only: binding to a port, accepting a
//! connection. Things that mean the process should probably stop.

use std::fmt;

/// The error type returned by tsu's fallible operations.
///
/// Not for 404s. Not for validation failures. Those are responses, not errors.
/// This is for infrastructure failures — port binding, connection acceptance.
/// The kind of failure where the right answer is probably to stop the process.
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
