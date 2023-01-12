use crate::{BoxError, Response, Validation};
use bytes::Bytes;
use http_body::Full;
use std::{error, fmt};
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

impl From<sqlx::Error> for Rejection {
    /// Converts to this type from the input type `sqlx::Error`.
    fn from(err: sqlx::Error) -> Self {
        InternalServerError(Box::new(err))
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

impl From<Rejection> for Response<http::StatusCode> {
    fn from(rejection: Rejection) -> Self {
        use Rejection::*;
        match rejection {
            BadRequest(validation) => {
                let mut res = Self::new(http::StatusCode::BAD_REQUEST);
                res.set_data(validation.into_map());
                res
            }
            Unauthorized(err) => {
                let mut res = Self::new(http::StatusCode::UNAUTHORIZED);
                res.set_message(err.to_string());
                res
            }
            Forbidden(err) => {
                let mut res = Self::new(http::StatusCode::FORBIDDEN);
                res.set_message(err.to_string());
                res
            }
            NotFound(err) => {
                let mut res = Self::new(http::StatusCode::NOT_FOUND);
                res.set_message(err.to_string());
                res
            }
            MethodNotAllowed(err) => {
                let mut res = Self::new(http::StatusCode::METHOD_NOT_ALLOWED);
                res.set_message(err.to_string());
                res
            }
            Conflict(err) => {
                let mut res = Self::new(http::StatusCode::CONFLICT);
                res.set_message(err.to_string());
                res
            }
            InternalServerError(err) => {
                let mut res = Self::new(http::StatusCode::INTERNAL_SERVER_ERROR);
                res.set_message(err.to_string());
                res
            }
        }
    }
}

impl<'a> From<&'a Rejection> for Response<http::StatusCode> {
    fn from(rejection: &'a Rejection) -> Self {
        use Rejection::*;
        match rejection {
            BadRequest(validation) => {
                let mut res = Self::new(http::StatusCode::BAD_REQUEST);
                res.set_data(validation.clone().into_map());
                res
            }
            Unauthorized(err) => {
                let mut res = Self::new(http::StatusCode::UNAUTHORIZED);
                res.set_message(err.to_string());
                res
            }
            Forbidden(err) => {
                let mut res = Self::new(http::StatusCode::FORBIDDEN);
                res.set_message(err.to_string());
                res
            }
            NotFound(err) => {
                let mut res = Self::new(http::StatusCode::NOT_FOUND);
                res.set_message(err.to_string());
                res
            }
            MethodNotAllowed(err) => {
                let mut res = Self::new(http::StatusCode::METHOD_NOT_ALLOWED);
                res.set_message(err.to_string());
                res
            }
            Conflict(err) => {
                let mut res = Self::new(http::StatusCode::CONFLICT);
                res.set_message(err.to_string());
                res
            }
            InternalServerError(err) => {
                let mut res = Self::new(http::StatusCode::INTERNAL_SERVER_ERROR);
                res.set_message(err.to_string());
                res
            }
        }
    }
}

impl From<Rejection> for http::Response<Full<Bytes>> {
    fn from(rejection: Rejection) -> Self {
        Response::from(rejection).into()
    }
}

impl From<Rejection> for Response<http_types::StatusCode> {
    fn from(rejection: Rejection) -> Self {
        use Rejection::*;
        match rejection {
            BadRequest(validation) => {
                let mut res = Self::new(http_types::StatusCode::BadRequest);
                res.set_data(validation.into_map());
                res
            }
            Unauthorized(err) => {
                let mut res = Self::new(http_types::StatusCode::Unauthorized);
                res.set_message(err.to_string());
                res
            }
            Forbidden(err) => {
                let mut res = Self::new(http_types::StatusCode::Forbidden);
                res.set_message(err.to_string());
                res
            }
            NotFound(err) => {
                let mut res = Self::new(http_types::StatusCode::NotFound);
                res.set_message(err.to_string());
                res
            }
            MethodNotAllowed(err) => {
                let mut res = Self::new(http_types::StatusCode::MethodNotAllowed);
                res.set_message(err.to_string());
                res
            }
            Conflict(err) => {
                let mut res = Self::new(http_types::StatusCode::Conflict);
                res.set_message(err.to_string());
                res
            }
            InternalServerError(err) => {
                let mut res = Self::new(http_types::StatusCode::InternalServerError);
                res.set_message(err.to_string());
                res
            }
        }
    }
}

impl<'a> From<&'a Rejection> for Response<http_types::StatusCode> {
    fn from(rejection: &'a Rejection) -> Self {
        use Rejection::*;
        match rejection {
            BadRequest(validation) => {
                let mut res = Self::new(http_types::StatusCode::BadRequest);
                res.set_data(validation.clone().into_map());
                res
            }
            Unauthorized(err) => {
                let mut res = Self::new(http_types::StatusCode::Unauthorized);
                res.set_message(err.to_string());
                res
            }
            Forbidden(err) => {
                let mut res = Self::new(http_types::StatusCode::Forbidden);
                res.set_message(err.to_string());
                res
            }
            NotFound(err) => {
                let mut res = Self::new(http_types::StatusCode::NotFound);
                res.set_message(err.to_string());
                res
            }
            MethodNotAllowed(err) => {
                let mut res = Self::new(http_types::StatusCode::MethodNotAllowed);
                res.set_message(err.to_string());
                res
            }
            Conflict(err) => {
                let mut res = Self::new(http_types::StatusCode::Conflict);
                res.set_message(err.to_string());
                res
            }
            InternalServerError(err) => {
                let mut res = Self::new(http_types::StatusCode::InternalServerError);
                res.set_message(err.to_string());
                res
            }
        }
    }
}
