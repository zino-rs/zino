//! Type-erased errors with tracing functionalities.
use crate::SharedString;
use std::{error, fmt};

mod source;

pub use source::Source;

/// An error type backed by an allocation-optimized string.
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

    /// Wraps the error value with additional contextual message.
    #[inline]
    pub fn context(self, message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(self)),
        }
    }

    /// Returns the error message.
    #[inline]
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    /// Returns the error source.
    #[inline]
    pub fn source(&self) -> Option<&Error> {
        self.source.as_deref()
    }

    /// Returns an iterator of the source errors contained by `self`.
    #[inline]
    pub fn sources(&self) -> Source<'_> {
        Source::new(self)
    }

    /// Returns the lowest level source of `self`.
    ///
    /// The root source is the last error in the iterator produced by [`sources()`](Error::sources).
    #[inline]
    pub fn root_source(&self) -> Option<&Error> {
        self.sources().last()
    }
}

impl<E: error::Error> From<E> for Error {
    #[inline]
    fn from(err: E) -> Self {
        Self {
            message: err.to_string().into(),
            source: err.source().map(|err| Box::new(Self::new(err.to_string()))),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = &self.message;
        if let Some(source) = &self.source {
            let source = source.message();
            let root_source = self.root_source().map(|err| err.message());
            if root_source != Some(source) {
                tracing::error!(root_source, source, "{message}");
            } else {
                tracing::error!(root_source, "{message}");
            }
        } else {
            tracing::error!("{message}");
        }
        write!(f, "{message}")
    }
}
