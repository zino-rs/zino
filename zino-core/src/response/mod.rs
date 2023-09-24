//! Constructing responses and rejections.

use crate::{
    error::Error,
    extension::JsonValueExt,
    file::NamedFile,
    helper,
    request::{RequestContext, Validation},
    trace::{ServerTiming, TimingMetric, TraceContext},
    JsonValue, SharedString, Uuid,
};
use bytes::Bytes;
use etag::EntityTag;
use http::header::{self, HeaderName, HeaderValue};
use http_body::Full;
use serde::Serialize;
use serde_json::value::RawValue;
use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

mod rejection;
mod response_code;
mod webhook;

pub use rejection::{ExtractRejection, Rejection};
pub use response_code::ResponseCode;
pub use webhook::WebHook;

/// An HTTP status code.
pub type StatusCode = http::StatusCode;

/// An Http response with the body that consists of a single chunk.
pub type FullResponse = http::Response<Full<Bytes>>;

/// A function pointer of transforming the response data.
pub type DataTransformer = fn(data: &JsonValue) -> Result<Bytes, Error>;

/// An HTTP response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Response<S = StatusCode> {
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
    /// JSON data.
    #[serde(rename = "data")]
    #[serde(skip_serializing_if = "JsonValue::is_null")]
    json_data: JsonValue,
    /// Bytes data.
    #[serde(skip)]
    bytes_data: Bytes,
    /// Transformer of the response data.
    #[serde(skip)]
    data_transformer: Option<DataTransformer>,
    /// Content type.
    #[serde(skip)]
    content_type: Option<SharedString>,
    /// Trace context.
    #[serde(skip)]
    trace_context: Option<TraceContext>,
    /// Server timing.
    #[serde(skip)]
    server_timing: ServerTiming,
    /// Custom headers.
    #[serde(skip)]
    headers: Vec<(SharedString, String)>,
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
            json_data: JsonValue::Null,
            bytes_data: Bytes::new(),
            data_transformer: None,
            content_type: None,
            trace_context: None,
            server_timing: ServerTiming::new(),
            headers: Vec::new(),
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
            instance: (!success).then(|| ctx.instance().into()),
            success,
            message: None,
            start_time: ctx.start_time(),
            request_id: ctx.request_id(),
            data: None,
            json_data: JsonValue::Null,
            bytes_data: Bytes::new(),
            data_transformer: None,
            content_type: None,
            trace_context: None,
            server_timing: ServerTiming::new(),
            headers: Vec::new(),
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
    pub fn context<Ctx: RequestContext>(mut self, ctx: &Ctx) -> Self {
        self.instance = (!self.is_success()).then(|| ctx.instance().into());
        self.start_time = ctx.start_time();
        self.request_id = ctx.request_id();
        self.trace_context = Some(ctx.new_trace_context());
        self
    }

    /// Renders a template and sets it as the reponse data.
    #[cfg(feature = "view")]
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
                    Err(Error::new("invalid template data"))
                }
            });
        match result {
            Ok(raw_value) => {
                self.data = Some(raw_value);
                self.json_data = JsonValue::Null;
                self.bytes_data = Bytes::new();
                self.content_type = Some("text/html; charset=utf-8".into());
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
                self.json_data = JsonValue::Null;
                self.bytes_data = Bytes::new();
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
    pub fn set_instance(&mut self, instance: impl Into<SharedString>) {
        self.instance = Some(instance.into());
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
    pub fn set_error_message(&mut self, error: impl Into<Error>) {
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
            Ok(raw_value) => {
                self.data = Some(raw_value);
                self.json_data = JsonValue::Null;
                self.bytes_data = Bytes::new();
            }
            Err(err) => self.set_error_message(err),
        }
    }

    /// Sets the JSON data.
    #[inline]
    pub fn set_json_data(&mut self, data: impl Into<JsonValue>) {
        self.data = None;
        self.json_data = data.into();
        self.bytes_data = Bytes::new();
    }

    /// Sets the bytes data.
    #[inline]
    pub fn set_bytes_data(&mut self, bytes: impl Into<Bytes>) {
        self.data = None;
        self.json_data = JsonValue::Null;
        self.bytes_data = bytes.into();
    }

    /// Sets the response data for the validation.
    #[inline]
    pub fn set_validation_data(&mut self, validation: Validation) {
        match serde_json::value::to_raw_value(&validation.into_map()) {
            Ok(raw_value) => {
                self.data = Some(raw_value);
                self.json_data = JsonValue::Null;
                self.bytes_data = Bytes::new();
            }
            Err(err) => self.set_error_message(err),
        }
    }

    /// Sets a transformer for the response data.
    #[inline]
    pub fn set_data_transformer(&mut self, transformer: DataTransformer) {
        self.data_transformer = Some(transformer);
    }

    /// Sets the content type.
    ///
    /// # Note
    ///
    /// Currently, we have built-in support for the following values:
    ///
    /// - `application/json`
    /// - `application/jsonlines`
    /// - `application/msgpack`
    /// - `application/octet-stream`
    /// - `application/problem+json`
    /// - `application/x-www-form-urlencoded`
    /// - `text/csv`
    /// - `text/html`
    /// - `text/plain`
    #[inline]
    pub fn set_content_type(&mut self, content_type: impl Into<SharedString>) {
        self.content_type = Some(content_type.into());
    }

    /// Sets the response body as the form data.
    #[inline]
    pub fn set_form_response(&mut self, data: impl Into<JsonValue>) {
        self.set_json_data(data);
        self.set_content_type("application/x-www-form-urlencoded");
        self.set_data_transformer(|data| {
            let mut bytes = Vec::new();
            serde_qs::to_writer(&data, &mut bytes)?;
            Ok(bytes.into())
        });
    }

    /// Sets the response body as the JSON data.
    #[inline]
    pub fn set_json_response(&mut self, data: impl Into<JsonValue>) {
        self.set_json_data(data);
        self.set_data_transformer(|data| Ok(serde_json::to_vec(&data)?.into()));
    }

    /// Sets the response body as the JSON Lines data.
    #[inline]
    pub fn set_jsonlines_response(&mut self, data: impl Into<JsonValue>) {
        self.set_json_data(data);
        self.set_content_type("application/jsonlines; charset=utf-8");
        self.set_data_transformer(|data| Ok(data.to_jsonlines(Vec::new())?.into()));
    }

    /// Sets the response body as the MsgPack data.
    #[inline]
    pub fn set_msgpack_response(&mut self, data: impl Into<JsonValue>) {
        self.set_json_data(data);
        self.set_content_type("application/msgpack");
        self.set_data_transformer(|data| Ok(data.to_msgpack(Vec::new())?.into()));
    }

    /// Sets the response body as the CSV data.
    #[inline]
    pub fn set_csv_response(&mut self, data: impl Into<JsonValue>) {
        self.set_json_data(data);
        self.set_content_type("text/csv; charset=utf-8");
        self.set_data_transformer(|data| Ok(data.to_csv(Vec::new())?.into()));
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

    /// Records a server timing metric entry.
    #[inline]
    pub fn record_server_timing(
        &mut self,
        name: impl Into<SharedString>,
        description: Option<SharedString>,
        duration: Option<Duration>,
    ) {
        let metric = TimingMetric::new(name.into(), description, duration);
        self.server_timing.push(metric);
    }

    /// Inserts a custom header.
    #[inline]
    pub fn insert_header(&mut self, name: impl Into<SharedString>, value: impl ToString) {
        self.headers.push((name.into(), value.to_string()));
    }

    /// Gets a custome header with the given name.
    #[inline]
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value.as_str()))
    }

    /// Returns the status code as `u16`.
    #[inline]
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Returns `true` if the response is successful or `false` otherwise.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns `true` if the response has a request context.
    #[inline]
    pub fn has_context(&self) -> bool {
        self.trace_context.is_some() && !self.request_id.is_nil()
    }

    /// Returns the message.
    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.detail
            .as_ref()
            .or(self.message.as_ref())
            .map(|s| s.as_ref())
    }

    /// Returns the request ID.
    #[inline]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Returns the trace ID.
    #[inline]
    pub fn trace_id(&self) -> Uuid {
        if let Some(ref trace_context) = self.trace_context {
            Uuid::from_u128(trace_context.trace_id())
        } else {
            Uuid::nil()
        }
    }

    /// Returns the content type.
    #[inline]
    pub fn content_type(&self) -> &str {
        self.content_type.as_deref().unwrap_or_else(|| {
            if !self.bytes_data.is_empty() {
                "application/octet-stream"
            } else if self.is_success() {
                "application/json; charset=utf-8"
            } else {
                "application/problem+json; charset=utf-8"
            }
        })
    }

    /// Returns the custom headers.
    #[inline]
    pub fn headers(&self) -> &[(SharedString, String)] {
        &self.headers
    }

    /// Returns the trace context in the form `(traceparent, tracestate)`.
    pub fn trace_context(&self) -> (String, String) {
        if let Some(ref trace_context) = self.trace_context {
            (trace_context.traceparent(), trace_context.tracestate())
        } else {
            let mut trace_context = TraceContext::new();
            let span_id = trace_context.span_id();
            trace_context
                .trace_state_mut()
                .push("zino", format!("{span_id:x}"));
            (trace_context.traceparent(), trace_context.tracestate())
        }
    }

    /// Returns the server timing.
    #[inline]
    pub fn server_timing(&self) -> String {
        self.server_timing.to_string()
    }

    /// Reads the response into a byte buffer.
    pub fn read_bytes(&mut self) -> Result<Bytes, Error> {
        let bytes_opt = if !self.bytes_data.is_empty() {
            Some(self.bytes_data.clone())
        } else if let Some(transformer) = self.data_transformer.as_ref() {
            if !self.json_data.is_null() {
                Some(transformer(&self.json_data)?)
            } else {
                let data = serde_json::to_value(&self.data)?;
                Some(transformer(&data)?)
            }
        } else {
            None
        };
        if let Some(bytes) = bytes_opt {
            let etag = EntityTag::from_data(&bytes);
            self.insert_header("x-etag", etag);
            return Ok(bytes);
        }

        let content_type = self.content_type();
        let (bytes, etag_opt) = if crate::helper::check_json_content_type(content_type) {
            let (capacity, etag_opt) = if !self.json_data.is_null() {
                let data = serde_json::to_vec(&self.json_data)?;
                let etag = EntityTag::from_data(&data);
                (data.len() + 128, Some(etag))
            } else if let Some(data) = &self.data {
                let data = data.get().as_bytes();
                let etag = EntityTag::from_data(data);
                (data.len() + 128, Some(etag))
            } else {
                (128, None)
            };
            let mut bytes = Vec::with_capacity(capacity);
            serde_json::to_writer(&mut bytes, &self)?;
            (bytes, etag_opt)
        } else if let Some(data) = &self.data {
            let capacity = data.get().len();
            let value = serde_json::to_value(data)?;
            let bytes = if content_type.starts_with("text/csv") {
                value.to_csv(Vec::with_capacity(capacity))?
            } else if content_type.starts_with("application/jsonlines") {
                value.to_jsonlines(Vec::with_capacity(capacity))?
            } else if content_type.starts_with("application/msgpack") {
                value.to_msgpack(Vec::with_capacity(capacity))?
            } else if let JsonValue::String(s) = value {
                s.into_bytes()
            } else {
                data.to_string().into_bytes()
            };
            (bytes, None)
        } else if !self.json_data.is_null() {
            let value = &self.json_data;
            let bytes = if content_type.starts_with("text/csv") {
                value.to_csv(Vec::new())?
            } else if content_type.starts_with("application/jsonlines") {
                value.to_jsonlines(Vec::new())?
            } else if content_type.starts_with("application/msgpack") {
                value.to_msgpack(Vec::new())?
            } else if let JsonValue::String(s) = value {
                s.clone().into_bytes()
            } else {
                value.to_string().into_bytes()
            };
            (bytes, None)
        } else {
            (Vec::new(), None)
        };
        let etag = etag_opt.unwrap_or_else(|| EntityTag::from_data(&bytes));
        self.insert_header("x-etag", etag);
        Ok(bytes.into())
    }

    /// Gets the response time.
    ///
    /// # Note
    ///
    /// It should only be called when the response will finish.
    pub fn response_time(&self) -> Duration {
        let duration = self.start_time.elapsed();
        let labels = [("status_code", self.status_code().to_string())];
        metrics::decrement_gauge!("zino_http_requests_in_flight", 1.0);
        metrics::increment_counter!("zino_http_responses_total", &labels);
        metrics::histogram!(
            "zino_http_requests_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        );
        duration
    }

    /// Sends a file to the client.
    pub fn send_file(&mut self, file: NamedFile) {
        let mut displayed_inline = false;
        if let Some(content_type) = file.content_type() {
            displayed_inline = helper::displayed_inline(content_type);
            self.set_content_type(content_type.to_string());
        }
        if let Some(file_name) = file.file_name() && !displayed_inline {
            self.insert_header(
                "content-disposition",
                format!(r#"attachment; filename="{file_name}""#),
            );
        }
        self.set_bytes_data(Bytes::from(file));
    }

    /// Consumes `self` and returns the custom headers.
    pub fn finalize(mut self) -> Vec<(SharedString, String)> {
        let request_id = self.request_id();
        if !request_id.is_nil() {
            self.insert_header("x-request-id", request_id.to_string());
        }

        let (traceparent, tracestate) = self.trace_context();
        self.insert_header("traceparent", traceparent);
        self.insert_header("tracestate", tracestate);

        let duration = self.response_time();
        self.record_server_timing("total", None, Some(duration));
        self.insert_header("server-timing", self.server_timing());

        self.headers
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

impl<S: ResponseCode> From<Response<S>> for FullResponse {
    fn from(mut response: Response<S>) -> Self {
        let mut res = match response.read_bytes() {
            Ok(data) => http::Response::builder()
                .status(response.status_code())
                .header(header::CONTENT_TYPE, response.content_type())
                .body(Full::from(data))
                .unwrap_or_default(),
            Err(err) => http::Response::builder()
                .status(S::INTERNAL_SERVER_ERROR.status_code())
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(Full::from(err.to_string()))
                .unwrap_or_default(),
        };

        for (key, value) in response.finalize() {
            if let Ok(header_name) = HeaderName::try_from(key.as_ref()) &&
                let Ok(header_value) = HeaderValue::try_from(value)
            {
                res.headers_mut().insert(header_name, header_value);
            }
        }

        res
    }
}
