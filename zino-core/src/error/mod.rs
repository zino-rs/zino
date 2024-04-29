//! Type-erased errors with tracing functionalities.
use crate::SharedString;
use std::{any::Any, error, fmt};

mod source;

use source::Source;

/// An error type backed by an allocation-optimized string.
#[derive(Debug)]
pub struct Error {
    /// Error message.
    message: SharedString,
    /// Error source.
    source: Option<Box<Error>>,
    /// Error context.
    context: Option<Box<dyn Any + Send>>,
}

impl Clone for Error {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            source: self.source.clone(),
            context: None,
        }
    }
}

impl PartialEq for Error {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.source == other.source
    }
}

impl Error {
    /// Creates a new instance with the supplied message.
    #[inline]
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: None,
            context: None,
        }
    }

    /// Creates a new instance with the supplied message and the error source.
    #[inline]
    pub fn with_source(message: impl Into<SharedString>, source: impl Into<Error>) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source.into())),
            context: None,
        }
    }

    /// Creates a new instance from [`std::error::Error`] by discarding the context.
    #[inline]
    pub fn from_error(err: impl error::Error) -> Self {
        Self {
            message: err.to_string().into(),
            source: err.source().map(|err| Box::new(Self::new(err.to_string()))),
            context: None,
        }
    }

    /// Wraps the error value with additional contextual message.
    #[inline]
    pub fn wrap(self, message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(self)),
            context: None,
        }
    }

    /// Sets a context for the error.
    #[inline]
    pub fn set_context<T: Send + 'static>(&mut self, context: T) {
        self.context = Some(Box::new(context));
    }

    /// Gets a reference to the context of the error.
    #[inline]
    pub fn get_context<T: Send + 'static>(&self) -> Option<&T> {
        self.context
            .as_ref()
            .and_then(|ctx| ctx.downcast_ref::<T>())
    }

    /// Gets a mutable reference to the context of the error.
    #[inline]
    pub fn get_context_mut<T: Send + 'static>(&mut self) -> Option<&mut T> {
        self.context
            .as_mut()
            .and_then(|ctx| ctx.downcast_mut::<T>())
    }

    /// Takes the context out of the error.
    #[inline]
    pub fn take_context<T: Send + 'static>(&mut self) -> Option<Box<T>> {
        self.context.take().and_then(|ctx| ctx.downcast::<T>().ok())
    }

    /// Returns `true` if the error has a context with type `T`.
    #[inline]
    pub fn has_context<T: Send + 'static>(&self) -> bool {
        self.context.as_ref().is_some_and(|ctx| ctx.is::<T>())
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
    pub fn sources(&self) -> impl Iterator<Item = &Error> {
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

impl<E: error::Error + Send + 'static> From<E> for Error {
    #[inline]
    fn from(err: E) -> Self {
        Self {
            message: err.to_string().into(),
            source: err.source().map(|err| Box::new(Self::new(err.to_string()))),
            context: Some(Box::new(err)),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self.message();
        if let Some(source) = &self.source {
            let source = source.message();
            let root_source = self.root_source().map(|err| err.message());
            if root_source != Some(source) {
                tracing::error!(root_source, source, message);
            } else {
                tracing::error!(root_source, message);
            }
        } else {
            tracing::error!(message);
        }
        write!(f, "{message}")
    }
}

/// Emits a `tracing::Event` at the warn level and returns early with an [`Error`].
#[macro_export]
macro_rules! bail {
    ($message:literal $(,)?) => {{
        tracing::warn!($message);
        return Err(Error::new($message));
    }};
    ($err:expr $(,)?) => {{
        tracing::warn!($err);
        return Err(Error::from($err));
    }};
    ($fmt:expr, $($arg:tt)+) => {{
        let message = format!($fmt, $($arg)+);
        tracing::warn!(message);
        return Err(Error::new(message));
    }};
}

/// Emits a `tracing::Event` at the warn level and constructs an [`Error`].
#[macro_export]
macro_rules! warn {
    ($message:literal $(,)?) => {{
        tracing::warn!($message);
        Error::new($message)
    }};
    ($err:expr $(,)?) => {{
        tracing::warn!($err);
        Error::from($err)
    }};
    ($fmt:expr, $($arg:tt)+) => {{
        let message = format!($fmt, $($arg)+);
        tracing::warn!(message);
        Error::new(message)
    }};
}
