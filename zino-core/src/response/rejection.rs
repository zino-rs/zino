use self::RejectionKind::*;
use super::Response;
use crate::{
    request::{Context, RequestContext, Validation},
    trace::TraceContext,
    BoxError,
};
use bytes::Bytes;
use http::StatusCode;
use http_body::Full;

/// A rejection response type.
#[derive(Debug)]
pub struct Rejection<'a> {
    /// Rejection kind.
    kind: RejectionKind,
    /// Optional context.
    context: Option<&'a Context>,
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

impl<'a> Rejection<'a> {
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
    pub fn unauthorized(err: impl Into<BoxError>) -> Self {
        Self {
            kind: Unauthorized(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `Forbidden` rejection.
    #[inline]
    pub fn forbidden(err: impl Into<BoxError>) -> Self {
        Self {
            kind: Forbidden(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `NotFound` rejection.
    #[inline]
    pub fn not_found(err: impl Into<BoxError>) -> Self {
        Self {
            kind: NotFound(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `MethodNotAllowed` rejection.
    #[inline]
    pub fn method_not_allowed(err: impl Into<BoxError>) -> Self {
        Self {
            kind: MethodNotAllowed(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `Conflict` rejection.
    #[inline]
    pub fn conflict(err: impl Into<BoxError>) -> Self {
        Self {
            kind: Conflict(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates an `InternalServerError` rejection.
    #[inline]
    pub fn internal_server_error(err: impl Into<BoxError>) -> Self {
        Self {
            kind: InternalServerError(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Provides the request context for the rejection.
    #[inline]
    pub fn provide_context<T: RequestContext>(mut self, ctx: &'a T) -> Self {
        self.context = ctx.get_context();
        self.trace_context = Some(ctx.new_trace_context());
        self
    }
}

impl<'a> From<Rejection<'a>> for http::Response<Full<Bytes>> {
    fn from(rejection: Rejection<'a>) -> Self {
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
        res.into()
    }
}

/// Trait for extracting rejections.
pub trait ExtractRejection<'a, T> {
    /// Extracts a rejection.
    fn extract(self) -> Result<T, Rejection<'a>>;

    /// Extracts a rejection with the request context.
    #[inline]
    fn extract_with_context<Ctx: RequestContext>(self, ctx: &'a Ctx) -> Result<T, Rejection<'a>>
    where
        Self: Sized,
    {
        self.extract()
            .map_err(|rejection| rejection.provide_context(ctx))
    }
}

impl<'a, T> ExtractRejection<'a, T> for Result<T, Validation> {
    #[inline]
    fn extract(self) -> Result<T, Rejection<'a>> {
        self.map_err(Rejection::bad_request)
    }
}

impl<'a, T, E: Into<BoxError>> ExtractRejection<'a, T> for Result<T, E> {
    #[inline]
    fn extract(self) -> Result<T, Rejection<'a>> {
        self.map_err(Rejection::internal_server_error)
    }
}

impl<'a, T, E: Into<BoxError>> ExtractRejection<'a, T> for Result<Option<T>, E> {
    #[inline]
    fn extract(self) -> Result<T, Rejection<'a>> {
        self.map_err(Rejection::internal_server_error)?
            .ok_or_else(|| Rejection::not_found("resource does not exit"))
    }
}
