//! Constructing responses and rejections.

use crate::{
    request::{RequestContext, Validation},
    trace::{ServerTiming, TimingMetric, TraceContext},
    BoxError, SharedString, Uuid,
};
use bytes::Bytes;
use http::header::{self, HeaderValue};
use http_body::Full;
use serde::Serialize;
use serde_json::value::{RawValue, Value};
use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

mod rejection;
mod response_code;

pub use rejection::{ExtractRejection, Rejection};
pub use response_code::ResponseCode;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Box<RawValue>>,
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
            data: None,
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
    pub fn with_context<Ctx: RequestContext>(code: S, ctx: &Ctx) -> Self {
        let success = code.is_success();
        let message = code.message();
        let mut res = Self {
            type_uri: code.type_uri(),
            title: code.title(),
            status_code: code.status_code(),
            error_code: code.error_code(),
            detail: None,
            instance: (!success).then(|| ctx.request_path().to_owned().into()),
            success,
            message: None,
            start_time: ctx.start_time(),
            request_id: ctx.request_id(),
            data: None,
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
        res.trace_context = Some(ctx.new_trace_context());
        res
    }

    /// Provides the request context for the response.
    pub fn provide_context<Ctx: RequestContext>(mut self, ctx: &Ctx) -> Self {
        self.instance = (!self.is_success()).then(|| ctx.request_path().to_owned().into());
        self.start_time = ctx.start_time();
        self.request_id = ctx.request_id();
        self.trace_context = Some(ctx.new_trace_context());
        self
    }

    #[cfg(feature = "view")]
    /// Renders a template and sets it as the reponse data.
    pub fn render<T: Serialize>(mut self, template_name: &str, data: T) -> Self {
        let result = serde_json::to_value(data)
            .map_err(|err| err.into())
            .and_then(|mut value| {
                if let Some(data) = value.as_object_mut() {
                    let mut map = crate::Map::new();
                    map.append(data);
                    crate::view::render(template_name, map).and_then(|data| {
                        serde_json::value::to_raw_value(&data).map_err(|err| err.into())
                    })
                } else {
                    Err("invalid template data".into())
                }
            });
        match result {
            Ok(raw_value) => {
                self.data = Some(raw_value);
                self.content_type = Some("text/html".into());
            }
            Err(err) => {
                let code = S::INTERNAL_SERVER_ERROR;
                self.type_uri = code.type_uri();
                self.title = code.title();
                self.status_code = code.status_code();
                self.error_code = code.error_code();
                self.success = false;
                self.detail = Some(err.to_string().into());
                self.message = None;
                self.data = None;
            }
        }
        self
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

    /// Sets a URI reference that identifies the specific occurrence of the problem.
    #[inline]
    pub fn set_instance(&mut self, instance: Option<SharedString>) {
        self.instance = instance;
    }

    /// Sets the message. If the response is not successful,
    /// it should be a human-readable explanation specific to this occurrence of the problem.
    pub fn set_message(&mut self, message: impl Into<SharedString>) {
        let message = message.into();
        if self.is_success() {
            self.detail = None;
            self.message = Some(message);
        } else {
            self.detail = Some(message);
            self.message = None;
        }
    }

    /// Sets the error message.
    pub fn set_error_message(&mut self, error: impl Into<BoxError>) {
        let message = error.into().to_string().into();
        if self.is_success() {
            self.detail = None;
            self.message = Some(message);
        } else {
            self.detail = Some(message);
            self.message = None;
        }
    }

    /// Sets the response data.
    #[inline]
    pub fn set_data<T: ?Sized + Serialize>(&mut self, data: &T) {
        match serde_json::value::to_raw_value(data) {
            Ok(raw_value) => self.data = Some(raw_value),
            Err(err) => self.set_error_message(err),
        }
    }

    /// Sets the response data for the validation.
    #[inline]
    pub fn set_validation_data(&mut self, validation: Validation) {
        match serde_json::value::to_raw_value(&validation.into_map()) {
            Ok(raw_value) => self.data = Some(raw_value),
            Err(err) => self.set_error_message(err),
        }
    }

    /// Sets the content type.
    #[inline]
    pub fn set_content_type(&mut self, content_type: impl Into<SharedString>) {
        self.content_type = Some(content_type.into());
    }

    /// Sets the request ID.
    #[inline]
    pub(crate) fn set_request_id(&mut self, request_id: Uuid) {
        self.request_id = request_id;
    }

    /// Sets the trace context from headers.
    #[inline]
    pub(crate) fn set_trace_context(&mut self, trace_context: Option<TraceContext>) {
        self.trace_context = trace_context;
    }

    /// Sets the start time.
    #[inline]
    pub(crate) fn set_start_time(&mut self, start_time: Instant) {
        self.start_time = start_time;
    }

    /// Enables the MessagePack encoding for the response data.
    #[inline]
    pub fn enable_msgpack_encoding(&mut self) {
        self.content_type = Some("application/msgpack".into());
    }

    /// Records a server timing metric entry.
    pub fn record_server_timing(
        &mut self,
        name: impl Into<SharedString>,
        description: Option<SharedString>,
        duration: Option<Duration>,
    ) {
        let metric = TimingMetric::new(name.into(), description, duration);
        self.server_timing.push(metric);
    }

    /// Returns `true` if the response is successful or `false` otherwise.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns the request ID.
    #[inline]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Returns the trace ID.
    pub fn trace_id(&self) -> Uuid {
        if let Some(ref trace_context) = self.trace_context {
            Uuid::from_u128(trace_context.trace_id())
        } else {
            Uuid::nil()
        }
    }
}

impl<S: ResponseCode> Default for Response<S> {
    #[inline]
    fn default() -> Self {
        Self::new(S::OK)
    }
}

impl<S: ResponseCode> From<Validation> for Response<S> {
    fn from(validation: Validation) -> Self {
        if validation.is_success() {
            Self::new(S::OK)
        } else {
            let mut res = Self::new(S::BAD_REQUEST);
            res.set_validation_data(validation);
            res
        }
    }
}

impl<S: ResponseCode> From<Response<S>> for http::Response<Full<Bytes>> {
    fn from(mut response: Response<S>) -> Self {
        let status_code = response.status_code;
        let mut res = if let Some(ref content_type) = response.content_type {
            if let Some(data) = &response.data {
                let result = serde_json::to_value(data)
                    .map_err(|err| err.to_string())
                    .and_then(|data| match data {
                        Value::String(s) => Ok(s.into_bytes()),
                        Value::Array(vec) => {
                            if content_type.starts_with("application/msgpack") {
                                rmp_serde::to_vec(&vec).map_err(|err| err.to_string())
                            } else {
                                Ok(Value::Array(vec).to_string().into_bytes())
                            }
                        }
                        Value::Object(map) => {
                            if content_type.starts_with("application/msgpack") {
                                rmp_serde::to_vec(&map).map_err(|err| err.to_string())
                            } else {
                                Ok(Value::Object(map).to_string().into_bytes())
                            }
                        }
                        _ => Err(data.to_string()),
                    });
                match result {
                    Ok(data) => http::Response::builder()
                        .status(status_code)
                        .header(header::CONTENT_TYPE, content_type.as_ref())
                        .body(Full::from(data))
                        .unwrap_or_default(),
                    Err(err) => http::Response::builder()
                        .status(S::INTERNAL_SERVER_ERROR.status_code())
                        .header(header::CONTENT_TYPE, content_type.as_ref())
                        .body(Full::from(err))
                        .unwrap_or_default(),
                }
            } else {
                http::Response::builder()
                    .status(status_code)
                    .header(header::CONTENT_TYPE, content_type.as_ref())
                    .body(Full::default())
                    .unwrap_or_default()
            }
        } else {
            match serde_json::to_vec(&response) {
                Ok(bytes) => {
                    let content_type = if response.is_success() {
                        "application/json"
                    } else {
                        "application/problem+json"
                    };
                    http::Response::builder()
                        .status(status_code)
                        .header(header::CONTENT_TYPE, content_type)
                        .body(Full::from(bytes))
                        .unwrap_or_default()
                }
                Err(err) => http::Response::builder()
                    .status(S::INTERNAL_SERVER_ERROR.status_code())
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Full::from(err.to_string()))
                    .unwrap_or_default(),
            }
        };
        let (traceparent, tracestate) = if let Some(ref trace_context) = response.trace_context {
            (trace_context.traceparent(), trace_context.tracestate())
        } else {
            let mut trace_context = TraceContext::new();
            let span_id = trace_context.span_id();
            trace_context
                .trace_state_mut()
                .push("zino", format!("{span_id:x}"));
            (trace_context.traceparent(), trace_context.tracestate())
        };
        if let Ok(header_value) = HeaderValue::try_from(traceparent) {
            res.headers_mut().insert("traceparent", header_value);
        }
        if let Ok(header_value) = HeaderValue::try_from(tracestate) {
            res.headers_mut().insert("tracestate", header_value);
        }

        let duration = response.start_time.elapsed();
        response.record_server_timing("total", None, Some(duration));
        if let Ok(header_value) = HeaderValue::try_from(response.server_timing.to_string()) {
            res.headers_mut().insert("server-timing", header_value);
        }

        let request_id = response.request_id;
        if !request_id.is_nil() {
            if let Ok(header_value) = HeaderValue::try_from(request_id.to_string()) {
                res.headers_mut().insert("x-request-id", header_value);
            }
        }

        // Emit metrics.
        let labels = [("status_code", status_code.to_string())];
        metrics::decrement_gauge!("zino_http_requests_in_flight", 1.0);
        metrics::increment_counter!("zino_http_responses_total", &labels);
        metrics::histogram!(
            "zino_http_requests_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        );

        res
    }
}
