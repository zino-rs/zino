use crate::{request::Validation, response::Response, BoxError};
use bytes::Bytes;
use http::StatusCode;
use http_body::Full;
use Rejection::*;

/// A rejection response type.
#[derive(Debug)]
#[non_exhaustive]
pub enum Rejection {
    /// 400 Bad Request
    BadRequest(Validation),
    /// 401 Unauthorized
    Unauthorized(BoxError),
    /// 403 Forbidden
    Forbidden(BoxError),
    /// 404 NotFound
    NotFound(BoxError),
    /// 405 Method Not Allowed
    MethodNotAllowed(BoxError),
    /// 409 Conflict
    Conflict(BoxError),
    /// 500 Internal Server Error
    InternalServerError(BoxError),
}

impl Rejection {
    /// Creates an `Unauthorized` rejection.
    #[inline]
    pub fn unauthorized(err: impl Into<BoxError>) -> Self {
        Unauthorized(err.into())
    }

    /// Creates a `Forbidden` rejection.
    #[inline]
    pub fn forbidden(err: impl Into<BoxError>) -> Self {
        Forbidden(err.into())
    }

    /// Creates a `NotFound` rejection.
    #[inline]
    pub fn not_found(err: impl Into<BoxError>) -> Self {
        NotFound(err.into())
    }

    /// Creates a `MethodNotAllowed` rejection.
    #[inline]
    pub fn method_not_allowed(err: impl Into<BoxError>) -> Self {
        MethodNotAllowed(err.into())
    }

    /// Creates a `Conflict` rejection.
    #[inline]
    pub fn conflict(err: impl Into<BoxError>) -> Self {
        Conflict(err.into())
    }

    /// Creates an `InternalServerError` rejection.
    #[inline]
    pub fn internal_server_error(err: impl Into<BoxError>) -> Self {
        InternalServerError(err.into())
    }
}

impl From<Validation> for Rejection {
    #[inline]
    fn from(validation: Validation) -> Self {
        BadRequest(validation)
    }
}

impl From<BoxError> for Rejection {
    #[inline]
    fn from(err: BoxError) -> Self {
        InternalServerError(err)
    }
}

impl From<Rejection> for http::Response<Full<Bytes>> {
    fn from(rejection: Rejection) -> Self {
        match rejection {
            BadRequest(validation) => {
                let mut res = Response::new(StatusCode::BAD_REQUEST);
                res.set_validation_data(validation);
                res.into()
            }
            Unauthorized(err) => {
                let mut res = Response::new(StatusCode::UNAUTHORIZED);
                res.set_error_message(err);
                res.into()
            }
            Forbidden(err) => {
                let mut res = Response::new(StatusCode::FORBIDDEN);
                res.set_error_message(err);
                res.into()
            }
            NotFound(err) => {
                let mut res = Response::new(StatusCode::NOT_FOUND);
                res.set_error_message(err);
                res.into()
            }
            MethodNotAllowed(err) => {
                let mut res = Response::new(StatusCode::METHOD_NOT_ALLOWED);
                res.set_error_message(err);
                res.into()
            }
            Conflict(err) => {
                let mut res = Response::new(StatusCode::CONFLICT);
                res.set_error_message(err);
                res.into()
            }
            InternalServerError(err) => {
                let mut res = Response::new(StatusCode::INTERNAL_SERVER_ERROR);
                res.set_error_message(err);
                res.into()
            }
        }
    }
}

/// Trait for extracting rejections.
pub trait ExtractRejection<T> {
    /// Extracs a rejection.
    fn extract_rejection(self) -> Result<T, Rejection>;
}

impl<T> ExtractRejection<T> for Result<T, Validation> {
    #[inline]
    fn extract_rejection(self) -> Result<T, Rejection> {
        self.map_err(BadRequest)
    }
}

impl<T, E: Into<BoxError>> ExtractRejection<T> for Result<T, E> {
    #[inline]
    fn extract_rejection(self) -> Result<T, Rejection> {
        self.map_err(|err| InternalServerError(err.into()))
    }
}

impl<T, E: Into<BoxError>> ExtractRejection<T> for Result<Option<T>, E> {
    #[inline]
    fn extract_rejection(self) -> Result<T, Rejection> {
        self.map_err(|err| InternalServerError(err.into()))?
            .ok_or_else(|| Rejection::not_found("resource does not exit"))
    }
}
