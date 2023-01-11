use crate::{RequestContext, SharedString, Uuid, Validation};
use bytes::Bytes;
use http::{
    self,
    header::{self, HeaderName, HeaderValue},
};
use http_body::Full;
use http_types::trace::{Metric, ServerTiming, TraceContext};
use serde::Serialize;
use serde_json::Value;
use std::{
    borrow::{Borrow, Cow},
    marker::PhantomData,
    time::{Duration, Instant},
};

mod rejection;

// Reexports.
pub use rejection::Rejection;

/// Response code.
/// See [Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807).
pub trait ResponseCode {
    /// 200 Ok.
    const OK: Self;

    /// Status code.
    fn status_code(&self) -> u16;

    /// Error code.
    fn error_code(&self) -> Option<SharedString>;

    /// Returns `true` if the response is successful.
    fn is_success(&self) -> bool;

    /// A URI reference that identifies the problem type.
    /// For successful response, it should be `None`.
    fn type_uri(&self) -> Option<SharedString>;

    /// A short, human-readable summary of the problem type.
    /// For successful response, it should be `None`.
    fn title(&self) -> Option<SharedString>;

    /// A context-specific descriptive message. If the response is not successful,
    /// it should be a human-readable explanation specific to this occurrence of the problem.
    fn message(&self) -> Option<SharedString>;
}

/// An HTTP response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Response<S> {
    /// A URI reference that identifies the problem type.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    type_uri: Option<SharedString>,
    /// A short, human-readable summary of the problem type.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<SharedString>,
    /// Status code.
    #[serde(rename = "status")]
    status_code: u16,
    /// Error code.
    #[serde(rename = "error")]
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<SharedString>,
    /// A human-readable explanation specific to this occurrence of the problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<SharedString>,
    /// A URI reference that identifies the specific occurrence of the problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<SharedString>,
    /// Indicates the response is successful or not.
    success: bool,
    /// A context-specific descriptive message for successful response.
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<SharedString>,
    /// Start time.
    #[serde(skip)]
    start_time: Instant,
    /// Request ID.
    #[serde(skip_serializing_if = "Uuid::is_nil")]
    request_id: Uuid,
    /// Response data.
    #[serde(skip_serializing_if = "Value::is_null")]
    data: Value,
    /// Content type.
    #[serde(skip)]
    content_type: Option<SharedString>,
    /// Trace context.
    #[serde(skip)]
    trace_context: Option<TraceContext>,
    /// Server timing.
    #[serde(skip)]
    server_timing: ServerTiming,
    /// Phantom type of response code.
    #[serde(skip)]
    phantom: PhantomData<S>,
}

impl<S: ResponseCode> Response<S> {
    /// Creates a new instance.
    pub fn new(code: S) -> Self {
        let success = code.is_success();
        let message = code.message();
        let mut res = Self {
            type_uri: code.type_uri(),
            title: code.title(),
            status_code: code.status_code(),
            error_code: code.error_code(),
            detail: None,
            instance: None,
            success,
            message: None,
            start_time: Instant::now(),
            request_id: Uuid::nil(),
            data: Value::Null,
            content_type: None,
            trace_context: None,
            server_timing: ServerTiming::new(),
            phantom: PhantomData,
        };
        if success {
            res.message = message;
        } else {
            res.detail = message;
        }
        res
    }

    /// Creates a new instance with the request context.
    pub fn with_context<T: RequestContext>(code: S, ctx: &T) -> Self {
        let success = code.is_success();
        let message = code.message();
        let mut res = Self {
            type_uri: code.type_uri(),
            title: code.title(),
            status_code: code.status_code(),
            error_code: code.error_code(),
            detail: None,
            instance: (!success).then(|| ctx.request_path().to_string().into()),
            success,
            message: None,
            start_time: ctx.start_time(),
            request_id: ctx.request_id(),
            data: Value::Null,
            content_type: None,
            trace_context: None,
            server_timing: ServerTiming::new(),
            phantom: PhantomData,
        };
        if success {
            res.message = message;
        } else {
            res.detail = message;
        }
        res.trace_context = ctx.trace_context().map(|t| t.child());
        res
    }

    /// Sets the code.
    pub fn set_code(&mut self, code: S) {
        let success = code.is_success();
        let message = code.message();
        self.type_uri = code.type_uri();
        self.title = code.title();
        self.status_code = code.status_code();
        self.error_code = code.error_code();
        self.success = success;
        if success {
            self.detail = None;
            self.message = message;
        } else {
            self.detail = message;
            self.message = None;
        }
    }

    /// Sets the request context.
    pub fn set_context<T: RequestContext>(&mut self, ctx: &T) {
        self.instance = (!self.is_success()).then(|| ctx.request_path().to_string().into());
        self.start_time = ctx.start_time();
        self.request_id = ctx.request_id();
        self.trace_context = ctx.trace_context().map(|t| t.child());
    }

    /// Returns `true` if the response is successful or `false` otherwise.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Sets a URI reference that identifies the specific occurrence of the problem.
    pub fn set_instance(&mut self, instance: impl Into<Option<SharedString>>) {
        self.instance = instance.into();
    }

    /// Sets the message. If the response is not successful,
    /// it should be a human-readable explanation specific to this occurrence of the problem.
    pub fn set_message(&mut self, message: impl Into<SharedString>) {
        if self.is_success() {
            self.detail = None;
            self.message = Some(message.into());
        } else {
            self.detail = Some(message.into());
            self.message = None;
        }
    }

    /// Sets the request ID.
    #[inline]
    pub fn set_request_id(&mut self, request_id: Uuid) {
        self.request_id = request_id;
    }

    /// Returns the request ID.
    #[inline]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Sets the response data.
    #[inline]
    pub fn set_data(&mut self, data: impl Into<Value>) {
        self.data = data.into();
    }

    /// Sets the content type.
    #[inline]
    pub fn set_content_type(&mut self, content_type: impl Into<SharedString>) {
        self.content_type = Some(content_type.into());
    }

    /// Sets the trace context from headers.
    #[inline]
    pub fn set_trace_context(&mut self, trace_context: impl Into<Option<TraceContext>>) {
        self.trace_context = trace_context.into();
    }

    /// Returns the trace ID.
    pub fn trace_id(&self) -> Uuid {
        match self.trace_context {
            Some(ref trace_context) => Uuid::from_u128(trace_context.trace_id()),
            None => Uuid::nil(),
        }
    }

    /// Records a server timing entry.
    pub fn record_server_timing(
        &mut self,
        name: impl Into<String>,
        dur: impl Into<Option<Duration>>,
        desc: impl Into<Option<String>>,
    ) {
        match Metric::new(name.into(), dur.into(), desc.into()) {
            Ok(entry) => self.server_timing.push(entry),
            Err(err) => tracing::error!("{err}"),
        }
    }

    /// Sets the start time.
    #[inline]
    pub fn set_start_time(&mut self, start_time: Instant) {
        self.start_time = start_time;
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}

impl ResponseCode for http::StatusCode {
    const OK: Self = http::StatusCode::OK;

    #[inline]
    fn status_code(&self) -> u16 {
        self.as_u16()
    }

    #[inline]
    fn error_code(&self) -> Option<SharedString> {
        None
    }

    #[inline]
    fn is_success(&self) -> bool {
        self.is_success()
    }

    #[inline]
    fn type_uri(&self) -> Option<SharedString> {
        None
    }

    #[inline]
    fn title(&self) -> Option<SharedString> {
        if self.is_success() {
            None
        } else {
            self.canonical_reason().map(Cow::Borrowed)
        }
    }

    #[inline]
    fn message(&self) -> Option<SharedString> {
        if self.is_success() {
            self.canonical_reason().map(Cow::Borrowed)
        } else {
            None
        }
    }
}

impl Default for Response<http::StatusCode> {
    #[inline]
    fn default() -> Self {
        Self::new(http::StatusCode::OK)
    }
}

impl From<Validation> for Response<http::StatusCode> {
    fn from(validation: Validation) -> Self {
        if validation.is_success() {
            Self::new(http::StatusCode::OK)
        } else {
            let mut res = Self::new(http::StatusCode::BAD_REQUEST);
            res.set_data(validation.into_map());
            res
        }
    }
}

impl From<Response<http::StatusCode>> for http::Response<Full<Bytes>> {
    fn from(mut response: Response<http::StatusCode>) -> Self {
        let mut res = match response.content_type {
            Some(ref content_type) => match serde_json::to_vec(&response.data) {
                Ok(bytes) => http::Response::builder()
                    .status(response.status_code)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(content_type.borrow()).unwrap(),
                    )
                    .body(Full::from(bytes))
                    .unwrap_or_default(),
                Err(err) => http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Full::from(err.to_string()))
                    .unwrap_or_default(),
            },
            None => match serde_json::to_vec(&response) {
                Ok(bytes) => {
                    let content_type = if response.is_success() {
                        "application/json"
                    } else {
                        "application/problem+json"
                    };
                    http::Response::builder()
                        .status(response.status_code)
                        .header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
                        .body(Full::from(bytes))
                        .unwrap_or_default()
                }
                Err(err) => http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Full::from(err.to_string()))
                    .unwrap_or_default(),
            },
        };
        let trace_context = match response.trace_context {
            Some(ref trace_context) => trace_context.value(),
            None => TraceContext::new().value(),
        };
        res.headers_mut().insert(
            HeaderName::from_static("traceparent"),
            HeaderValue::from_str(trace_context.as_str()).unwrap(),
        );

        response.record_server_timing("total", response.start_time.elapsed(), None);
        res.headers_mut().insert(
            HeaderName::from_static("server-timing"),
            HeaderValue::from_str(response.server_timing.value().as_str()).unwrap(),
        );

        let request_id = response.request_id;
        if !request_id.is_nil() {
            res.headers_mut().insert(
                HeaderName::from_static("x-request-id"),
                HeaderValue::from_str(request_id.to_string().as_str()).unwrap(),
            );
        }
        res
    }
}

impl ResponseCode for http_types::StatusCode {
    const OK: Self = http_types::StatusCode::Ok;

    #[inline]
    fn status_code(&self) -> u16 {
        *self as u16
    }

    #[inline]
    fn error_code(&self) -> Option<SharedString> {
        None
    }

    #[inline]
    fn is_success(&self) -> bool {
        self.is_success()
    }

    #[inline]
    fn type_uri(&self) -> Option<SharedString> {
        None
    }

    #[inline]
    fn title(&self) -> Option<SharedString> {
        (!self.is_success()).then(|| self.canonical_reason().into())
    }

    #[inline]
    fn message(&self) -> Option<SharedString> {
        self.is_success().then(|| self.canonical_reason().into())
    }
}

impl Default for Response<http_types::StatusCode> {
    #[inline]
    fn default() -> Self {
        Self::new(http_types::StatusCode::Ok)
    }
}

impl From<Validation> for Response<http_types::StatusCode> {
    fn from(validation: Validation) -> Self {
        if validation.is_success() {
            Self::new(http_types::StatusCode::Ok)
        } else {
            let mut res = Self::new(http_types::StatusCode::BadRequest);
            res.set_data(validation.into_map());
            res
        }
    }
}
