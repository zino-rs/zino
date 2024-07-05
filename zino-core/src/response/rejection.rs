use self::RejectionKind::*;
use super::{Response, StatusCode};
use crate::{
    error::Error,
    request::{Context, RequestContext},
    trace::TraceContext,
    validation::Validation,
    warn, SharedString,
};

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
    /// 503 Service Unavailable
    ServiceUnavailable(Error),
}

impl Rejection {
    /// Creates a `400 Bad Request` rejection.
    #[inline]
    pub fn bad_request(validation: Validation) -> Self {
        Self {
            kind: BadRequest(validation),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `401 Unauthorized` rejection.
    #[inline]
    pub fn unauthorized(err: impl Into<Error>) -> Self {
        Self {
            kind: Unauthorized(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `403 Forbidden` rejection.
    #[inline]
    pub fn forbidden(err: impl Into<Error>) -> Self {
        Self {
            kind: Forbidden(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `404 Not Found` rejection.
    #[inline]
    pub fn not_found(err: impl Into<Error>) -> Self {
        Self {
            kind: NotFound(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `405 Method Not Allowed` rejection.
    #[inline]
    pub fn method_not_allowed(err: impl Into<Error>) -> Self {
        Self {
            kind: MethodNotAllowed(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `409 Conflict` rejection.
    #[inline]
    pub fn conflict(err: impl Into<Error>) -> Self {
        Self {
            kind: Conflict(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `500 Internal Server Error` rejection.
    #[inline]
    pub fn internal_server_error(err: impl Into<Error>) -> Self {
        Self {
            kind: InternalServerError(err.into()),
            context: None,
            trace_context: None,
        }
    }

    /// Creates a `503 Service Unavailable` rejection.
    #[inline]
    pub fn service_unavailable(err: impl Into<Error>) -> Self {
        Self {
            kind: ServiceUnavailable(err.into()),
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

    /// Creates a new instance from an error classified by the error message.
    pub fn from_error(err: impl Into<Error>) -> Self {
        fn inner(err: Error) -> Rejection {
            let message = err.message();
            if message.starts_with("401 Unauthorized") {
                Rejection::unauthorized(err)
            } else if message.starts_with("403 Forbidden") {
                Rejection::forbidden(err)
            } else if message.starts_with("404 Not Found") {
                Rejection::not_found(err)
            } else if message.starts_with("405 Method Not Allowed") {
                Rejection::method_not_allowed(err)
            } else if message.starts_with("409 Conflict") {
                Rejection::conflict(err)
            } else if message.starts_with("503 Service Unavailable") {
                Rejection::service_unavailable(err)
            } else {
                Rejection::internal_server_error(err)
            }
        }
        inner(err.into())
    }

    /// Creates a new instance with the error message.
    #[inline]
    pub fn with_message(message: impl Into<SharedString>) -> Self {
        Self::from_error(Error::new(message))
    }

    /// Provides the request context for the rejection.
    #[inline]
    pub fn context<T: RequestContext + ?Sized>(mut self, ctx: &T) -> Self {
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
            ServiceUnavailable(_) => 503,
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
            ServiceUnavailable(err) => {
                let mut res = Response::new(StatusCode::SERVICE_UNAVAILABLE);
                res.set_error_message(err);
                res
            }
        };
        if let Some(ctx) = rejection.context {
            res.set_instance(ctx.instance().to_owned());
            res.set_start_time(ctx.start_time());
            res.set_request_id(ctx.request_id());
        }
        res.set_trace_context(rejection.trace_context);
        res
    }
}

/// Trait for extracting rejections.
pub trait ExtractRejection<T> {
    /// Extracts a rejection with the request context.
    fn extract<Ctx: RequestContext>(self, ctx: &Ctx) -> Result<T, Rejection>;
}

impl<T> ExtractRejection<T> for Option<T> {
    #[inline]
    fn extract<Ctx: RequestContext>(self, ctx: &Ctx) -> Result<T, Rejection> {
        self.ok_or_else(|| Rejection::not_found(warn!("resource does not exist")).context(ctx))
    }
}

impl<T> ExtractRejection<T> for Result<T, Validation> {
    #[inline]
    fn extract<Ctx: RequestContext>(self, ctx: &Ctx) -> Result<T, Rejection> {
        self.map_err(|err| Rejection::bad_request(err).context(ctx))
    }
}

impl<T, E: Into<Error>> ExtractRejection<T> for Result<T, E> {
    #[inline]
    fn extract<Ctx: RequestContext>(self, ctx: &Ctx) -> Result<T, Rejection> {
        self.map_err(|err| Rejection::from_error(err).context(ctx))
    }
}

impl<T, E: Into<Error>> ExtractRejection<T> for Result<Option<T>, E> {
    #[inline]
    fn extract<Ctx: RequestContext>(self, ctx: &Ctx) -> Result<T, Rejection> {
        self.map_err(|err| Rejection::from_error(err).context(ctx))?
            .ok_or_else(|| Rejection::not_found(warn!("resource does not exist")).context(ctx))
    }
}

/// Returns early with a [`Rejection`].
#[macro_export]
macro_rules! reject {
    ($ctx:ident, $validation:expr $(,)?) => {{
        return Err(Rejection::bad_request($validation).context(&$ctx).into());
    }};
    ($ctx:ident, $key:literal, $message:literal $(,)?) => {{
        let err = Error::new($message);
        warn!("invalid value for `{}`: {}", $key, $message);
        return Err(Rejection::from_validation_entry($key, err).context(&$ctx).into());
    }};
    ($ctx:ident, $key:literal, $err:expr $(,)?) => {{
        return Err(Rejection::from_validation_entry($key, $err).context(&$ctx).into());
    }};
    ($ctx:ident, $kind:ident, $message:literal $(,)?) => {{
        let err = warn!($message);
        return Err(Rejection::$kind(err).context(&$ctx).into());
    }};
    ($ctx:ident, $kind:ident, $err:expr $(,)?) => {{
        return Err(Rejection::$kind($err).context(&$ctx).into());
    }};
    ($ctx:ident, $kind:ident, $fmt:expr, $($arg:tt)+) => {{
        let err = warn!($fmt, $($arg)+);
        return Err(Rejection::$kind(err).context(&$ctx).into());
    }};
}
