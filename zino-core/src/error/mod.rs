//! Type-erased errors with tracing functionalities.
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
    /// Creates a new instance with the supplied message.
    #[inline]
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a new instance with the supplied message and the error source.
    #[inline]
    pub fn with_source(message: impl Into<SharedString>, source: impl Into<Error>) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source.into())),
        }
    }

    /// Returns a new instance with the supplied message and `self` as the error source.
    #[inline]
    pub fn wrap(self, message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(self)),
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
        let message = &self.message;
        if let Some(source) = &self.source {
            tracing::error!(source = source.to_string(), "{message}");
            write!(f, "{message}: {source}")
        } else {
            tracing::error!("{message}");
            write!(f, "{message}")
        }
    }
}
