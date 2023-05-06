use self::RejectionKind::*;
use super::{FullResponse, Response};
use crate::{
    error::Error,
    request::{Context, RequestContext, Validation},
    trace::TraceContext,
    SharedString,
};
use http::StatusCode;

/// A rejection response type.
#[derive(Debug)]
pub struct Rejection {
    /// Rejection kind.
    kind: RejectionKind,
    /// Optional context.
    context: Option<Context>,
    /// Optional trace context.
    trace_context: Option<TraceContext>,
}

/// Rejection kind.
#[derive(Debug)]
#[non_exhaustive]
enum RejectionKind {
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
    /// Creates an `BadRequest` rejection.
    #[inline]
    pub fn bad_request(validation: Validation) -> Self {
        Self {
            kind: BadRequest(validation),
            context: None,
            trace_context: None,
        }
    }

    /// Creates an `Unauthorized` rejection.
    #[inline]
    pub fn unauthorized(err: impl Into<Error>) -> Self {
        Self {
            kind: Unauthorized(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `Forbidden` rejection.
    #[inline]
    pub fn forbidden(err: impl Into<Error>) -> Self {
        Self {
            kind: Forbidden(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `NotFound` rejection.
    #[inline]
    pub fn not_found(err: impl Into<Error>) -> Self {
        Self {
            kind: NotFound(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `MethodNotAllowed` rejection.
    #[inline]
    pub fn method_not_allowed(err: impl Into<Error>) -> Self {
        Self {
            kind: MethodNotAllowed(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `Conflict` rejection.
    #[inline]
    pub fn conflict(err: impl Into<Error>) -> Self {
        Self {
            kind: Conflict(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates an `InternalServerError` rejection.
    #[inline]
    pub fn internal_server_error(err: impl Into<Error>) -> Self {
        Self {
            kind: InternalServerError(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a new instance with the validation entry.
    #[inline]
    pub fn from_validation_entry(key: impl Into<SharedString>, err: impl Into<Error>) -> Self {
        let validation = Validation::from_entry(key, err);
        Self::bad_request(validation)
    }

    /// Provides the request context for the rejection.
    #[inline]
    pub fn provide_context<T: RequestContext + ?Sized>(mut self, ctx: &T) -> Self {
        self.context = ctx.get_context();
        self.trace_context = Some(ctx.new_trace_context());
        self
    }

    /// Returns the status code as `u16`.
    #[inline]
    pub fn status_code(&self) -> u16 {
        match &self.kind {
            BadRequest(_) => 400,
            Unauthorized(_) => 401,
            Forbidden(_) => 403,
            NotFound(_) => 404,
            MethodNotAllowed(_) => 405,
            Conflict(_) => 409,
            InternalServerError(_) => 500,
        }
    }
}

impl From<Rejection> for Response<StatusCode> {
    fn from(rejection: Rejection) -> Self {
        let mut res = match rejection.kind {
            BadRequest(validation) => {
                let mut res = Response::new(StatusCode::BAD_REQUEST);
                res.set_validation_data(validation);
                res
            }
            Unauthorized(err) => {
                let mut res = Response::new(StatusCode::UNAUTHORIZED);
                res.set_error_message(err);
                res
            }
            Forbidden(err) => {
                let mut res = Response::new(StatusCode::FORBIDDEN);
                res.set_error_message(err);
                res
            }
            NotFound(err) => {
                let mut res = Response::new(StatusCode::NOT_FOUND);
                res.set_error_message(err);
                res
            }
            MethodNotAllowed(err) => {
                let mut res = Response::new(StatusCode::METHOD_NOT_ALLOWED);
                res.set_error_message(err);
                res
            }
            Conflict(err) => {
                let mut res = Response::new(StatusCode::CONFLICT);
                res.set_error_message(err);
                res
            }
            InternalServerError(err) => {
                let mut res = Response::new(StatusCode::INTERNAL_SERVER_ERROR);
                res.set_error_message(err);
                res
            }
        };
        if let Some(ctx) = rejection.context {
            res.set_instance(Some(ctx.instance().to_owned().into()));
            res.set_start_time(ctx.start_time());
            res.set_request_id(ctx.request_id());
        }
        res.set_trace_context(rejection.trace_context);
        res
    }
}

impl From<Rejection> for FullResponse {
    #[inline]
    fn from(rejection: Rejection) -> Self {
        Response::from(rejection).into()
    }
}

/// Trait for extracting rejections.
pub trait ExtractRejection<T> {
    /// Extracts a rejection.
    fn extract(self) -> Result<T, Rejection>;

    /// Extracts a rejection with the request context.
    #[inline]
    fn extract_with_context<Ctx: RequestContext>(self, ctx: &Ctx) -> Result<T, Rejection>
    where
        Self: Sized,
    {
        self.extract()
            .map_err(|rejection| rejection.provide_context(ctx))
    }
}

impl<T> ExtractRejection<T> for Result<T, Validation> {
    #[inline]
    fn extract(self) -> Result<T, Rejection> {
        self.map_err(Rejection::bad_request)
    }
}

impl<T, E: Into<Error>> ExtractRejection<T> for Result<T, E> {
    #[inline]
    fn extract(self) -> Result<T, Rejection> {
        self.map_err(Rejection::internal_server_error)
    }
}

impl<T, E: Into<Error>> ExtractRejection<T> for Result<Option<T>, E> {
    #[inline]
    fn extract(self) -> Result<T, Rejection> {
        self.map_err(Rejection::internal_server_error)?
            .ok_or_else(|| Rejection::not_found(Error::new("resource does not exit")))
    }
}
