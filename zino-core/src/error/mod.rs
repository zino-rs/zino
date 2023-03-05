//! Type-erased errors with backtraces.
use crate::SharedString;
use std::{error, fmt};

/// A error type backed by an allocation-optimized string.
#[derive(Debug)]
pub struct Error {
    /// Error message.
    message: SharedString,
    /// Error source.
    source: Option<Box<Error>>,
}

impl Error {
    /// Creates a new instance with the error message.
    #[inline]
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Returns the source.
    #[inline]
    pub fn source(&self) -> Option<&Error> {
        self.source.as_deref()
    }
}

impl<E: error::Error + 'static> From<E> for Error {
    #[inline]
    fn from(err: E) -> Self {
        Self {
            message: err.to_string().into(),
            source: err.source().map(|err| Box::new(Self::new(err.to_string()))),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.message, f)
    }
}
