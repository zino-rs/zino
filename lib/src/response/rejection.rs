use crate::{Error, Validation};
use std::{error, fmt};
use Rejection::*;

/// A rejection response type.
#[derive(Debug)]
#[non_exhaustive]
pub enum Rejection {
    /// 400 Bad Request
    BadRequest(Validation),
    /// 401 Unauthorized
    Unauthorized(Error),
    /// 403 Forbidden
    Forbidden(Error),
    /// 404 NotFound
    NotFound(Error),
    /// 405 Method Not Allowed
    MethodNotAllowed(Error),
    /// 409 Conflict
    Conflict(Error),
    /// 500 Internal Server Error
    InternalServerError(Error),
}

impl Rejection {
    /// Creates a `BadRequest` rejection.
    #[inline]
    pub fn bad_request(validation: Validation) -> Self {
        BadRequest(validation)
    }

    /// Creates an `Unauthorized` rejection.
    #[inline]
    pub fn unauthorized(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Unauthorized(Box::new(err))
    }

    /// Creates a `Forbidden` rejection.
    #[inline]
    pub fn forbidden(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Forbidden(Box::new(err))
    }

    /// Creates a `NotFound` rejection.
    #[inline]
    pub fn not_found(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        NotFound(Box::new(err))
    }

    /// Creates a `MethodNotAllowed` rejection.
    #[inline]
    pub fn method_not_allowed(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        MethodNotAllowed(Box::new(err))
    }

    /// Creates a `Conflict` rejection.
    #[inline]
    pub fn conflict(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Conflict(Box::new(err))
    }

    /// Creates an `InternalServerError` rejection.
    #[inline]
    pub fn internal_server_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        InternalServerError(Box::new(err))
    }
}

impl From<Validation> for Rejection {
    fn from(validation: Validation) -> Self {
        BadRequest(validation)
    }
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadRequest(validation) => write!(f, "Bad Request: {validation}"),
            Unauthorized(err) => write!(f, "Unauthorized: {err}"),
            Forbidden(err) => write!(f, "Forbidden: {err}"),
            NotFound(err) => write!(f, "Not Found: {err}"),
            MethodNotAllowed(err) => write!(f, "Method Not Allowed: {err}"),
            Conflict(err) => write!(f, "Conflict: {err}"),
            InternalServerError(err) => write!(f, "Internal Server Error: {err}"),
        }
    }
}

impl error::Error for Rejection {}
