use crate::{request::Validation, response::Response, BoxError};
use bytes::Bytes;
use http::StatusCode;
use http_body::Full;
use std::error::Error;
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
    /// Creates a `BadRequest` rejection.
    #[inline]
    pub fn bad_request(validation: Validation) -> Self {
        BadRequest(validation)
    }

    /// Creates an `Unauthorized` rejection.
    #[inline]
    pub fn unauthorized(err: impl Error + Send + Sync + 'static) -> Self {
        Unauthorized(Box::new(err))
    }

    /// Creates a `Forbidden` rejection.
    #[inline]
    pub fn forbidden(err: impl Error + Send + Sync + 'static) -> Self {
        Forbidden(Box::new(err))
    }

    /// Creates a `NotFound` rejection.
    #[inline]
    pub fn not_found(err: impl Error + Send + Sync + 'static) -> Self {
        NotFound(Box::new(err))
    }

    /// Creates a `MethodNotAllowed` rejection.
    #[inline]
    pub fn method_not_allowed(err: impl Error + Send + Sync + 'static) -> Self {
        MethodNotAllowed(Box::new(err))
    }

    /// Creates a `Conflict` rejection.
    #[inline]
    pub fn conflict(err: impl Error + Send + Sync + 'static) -> Self {
        Conflict(Box::new(err))
    }

    /// Creates an `InternalServerError` rejection.
    #[inline]
    pub fn internal_server_error(err: impl Error + Send + Sync + 'static) -> Self {
        InternalServerError(Box::new(err))
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

impl From<sqlx::Error> for Rejection {
    /// Converts to this type from the input type `sqlx::Error`.
    #[inline]
    fn from(err: sqlx::Error) -> Self {
        InternalServerError(Box::new(err))
    }
}

impl From<Rejection> for Response<StatusCode> {
    fn from(rejection: Rejection) -> Self {
        use Rejection::*;
        match rejection {
            BadRequest(validation) => {
                let mut res = Self::new(StatusCode::BAD_REQUEST);
                res.set_validation_data(validation);
                res
            }
            Unauthorized(err) => {
                let mut res = Self::new(StatusCode::UNAUTHORIZED);
                res.set_error_message(err);
                res
            }
            Forbidden(err) => {
                let mut res = Self::new(StatusCode::FORBIDDEN);
                res.set_error_message(err);
                res
            }
            NotFound(err) => {
                let mut res = Self::new(StatusCode::NOT_FOUND);
                res.set_error_message(err);
                res
            }
            MethodNotAllowed(err) => {
                let mut res = Self::new(StatusCode::METHOD_NOT_ALLOWED);
                res.set_error_message(err);
                res
            }
            Conflict(err) => {
                let mut res = Self::new(StatusCode::CONFLICT);
                res.set_error_message(err);
                res
            }
            InternalServerError(err) => {
                let mut res = Self::new(StatusCode::INTERNAL_SERVER_ERROR);
                res.set_error_message(err);
                res
            }
        }
    }
}

impl From<Rejection> for http::Response<Full<Bytes>> {
    #[inline]
    fn from(rejection: Rejection) -> Self {
        Response::from(rejection).into()
    }
}
