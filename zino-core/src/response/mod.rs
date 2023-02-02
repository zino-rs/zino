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
use serde_json::Value;
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
            instance: (!success).then(|| ctx.request_path().to_owned().into()),
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
        res.trace_context = Some(ctx.new_trace_context());
        res
    }

    /// Provides the request context for the response.
    pub fn provide_context<T: RequestContext>(mut self, ctx: &T) -> Self {
        self.instance = (!self.is_success()).then(|| ctx.request_path().to_owned().into());
        self.start_time = ctx.start_time();
        self.request_id = ctx.request_id();
        self.trace_context = Some(ctx.new_trace_context());
        self
    }

    #[cfg(feature = "view")]
    /// Renders a template and sets it as the reponse data.
    pub fn render(mut self, template_name: &str) -> Self {
        if let Some(data) = self.data.as_object_mut() {
            let mut map = crate::Map::new();
            map.append(data);
            match crate::view::render(template_name, map) {
                Ok(data) => {
                    self.data = data.into();
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
                    self.data = Value::Null;
                }
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
    pub fn set_data(&mut self, data: impl Into<Value>) {
        self.data = data.into();
    }

    /// Sets the response data for the validation.
    pub fn set_validation_data(&mut self, validation: Validation) {
        self.data = validation.into_map().into();
    }

    /// Sets the content type.
    #[inline]
    pub fn set_content_type(&mut self, content_type: impl Into<SharedString>) {
        self.content_type = Some(content_type.into());
    }

    /// Records a server timing metric entry.
    pub fn record_server_timing(
        &mut self,
        name: impl Into<SharedString>,
        description: Option<SharedString>,
        duration: impl Into<Option<Duration>>,
    ) {
        let metric = TimingMetric::new(name.into(), description, duration.into());
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
        match self.trace_context {
            Some(ref trace_context) => Uuid::from_u128(trace_context.trace_id()),
            None => Uuid::nil(),
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
        let mut res = match response.content_type {
            Some(ref content_type) => {
                let ref data = response.data;
                if let Some(data) = data.as_str() {
                    http::Response::builder()
                        .status(status_code)
                        .header(header::CONTENT_TYPE, content_type.as_ref())
                        .body(Full::from(data.to_owned()))
                        .unwrap_or_default()
                } else {
                    match serde_json::to_vec(&data) {
                        Ok(bytes) => http::Response::builder()
                            .status(status_code)
                            .header(header::CONTENT_TYPE, content_type.as_ref())
                            .body(Full::from(bytes))
                            .unwrap_or_default(),
                        Err(err) => http::Response::builder()
                            .status(S::INTERNAL_SERVER_ERROR.status_code())
                            .header(header::CONTENT_TYPE, "text/plain")
                            .body(Full::from(err.to_string()))
                            .unwrap_or_default(),
                    }
                }
            }
            None => match serde_json::to_vec(&response) {
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
            },
        };
        let (traceparent, tracestate) = match response.trace_context {
            Some(ref trace_context) => (trace_context.traceparent(), trace_context.tracestate()),
            None => {
                let mut trace_context = TraceContext::new();
                let span_id = trace_context.span_id();
                trace_context
                    .trace_state_mut()
                    .push("zino", format!("{span_id:x}"));
                (trace_context.traceparent(), trace_context.tracestate())
            }
        };
        if let Ok(header_value) = HeaderValue::try_from(traceparent) {
            res.headers_mut().insert("traceparent", header_value);
        }
        if let Ok(header_value) = HeaderValue::try_from(tracestate) {
            res.headers_mut().insert("tracestate", header_value);
        }

        let duration = response.start_time.elapsed();
        response.record_server_timing("total", None, duration);
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
        metrics::decrement_gauge!("zino_http_requests_pending", 1.0);
        metrics::increment_counter!("zino_http_responses_total", &labels);
        metrics::histogram!(
            "zino_http_requests_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        );

        res
    }
}
