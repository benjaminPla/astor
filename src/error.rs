//! Unified error type.

use std::fmt;

/// The error type returned by tsu's fallible operations.
///
/// Application-level errors (404, 422, etc.) are expressed as HTTP
/// [`Response`](crate::Response) values, not as `Error`s. This type surfaces
/// infrastructure failures: binding to a port, accepting a connection, or an
/// unexpected protocol error from the underlying Hyper layer.
///
/// The inner variant is heap-allocated (`Box`) so that `Error` is always
/// pointer-sized regardless of which variant is active — a common pattern in
/// Rust error types that keeps function return types small.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

#[derive(Debug)]
enum ErrorKind {
    Io(std::io::Error),
    Hyper(hyper::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_ref() {
            ErrorKind::Io(e) => write!(f, "i/o error: {e}"),
            ErrorKind::Hyper(e) => write!(f, "hyper error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.0.as_ref() {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::Hyper(e) => Some(e),
        }
    }
}

// `From` impls let callers use `?` to convert standard errors into `Error`.

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(Box::new(ErrorKind::Io(e)))
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Self(Box::new(ErrorKind::Hyper(e)))
    }
}
