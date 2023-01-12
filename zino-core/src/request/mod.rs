use crate::{
    authentication::ParseTokenError, Authentication, CloudEvent, DateTime, Map, Model, Query,
    Rejection, Response, ResponseCode, SecurityToken, Subscription, Uuid,
};
use http::uri::Uri;
use http_types::{trace::TraceContext, Trailers};
use hyper::body::Bytes;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::{Duration, Instant};
use toml::value::Table;

mod context;
mod validation;

// Reexports.
pub use context::Context;
pub use validation::Validation;

/// Request context.
pub trait RequestContext {
    /// Returns a reference to the application config.
    fn config(&self) -> &Table;

    /// Returns a reference to the request scoped state data.
    fn state_data(&self) -> &Map;

    /// Returns a mutable reference to the request scoped state data.
    fn state_data_mut(&mut self) -> &mut Map;

    /// Gets a reference to the request context.
    fn get_context(&self) -> Option<&Context>;

    /// Gets an HTTP header.
    fn get_header(&self, key: &str) -> Option<&str>;

    /// Returns the request method.
    fn request_method(&self) -> &str;

    /// Returns the path in the router that matches the request.
    fn matched_path(&self) -> &str;

    /// Returns the original request URI regardless of nesting.
    fn original_uri(&self) -> &Uri;

    /// Concatenates buffers from the request body into a single `Bytes` asynchronously.
    async fn to_bytes(&mut self) -> Result<Bytes, Validation>;

    /// Attempts to send a message.
    fn try_send(&self, message: impl Into<CloudEvent>) -> Result<(), Rejection>;

    /// Creates a new request context.
    fn new_context(&self) -> Context {
        let request_id = self
            .get_header("x-request-id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(Uuid::new_v4());
        let trace_context = self.trace_context();
        let trace_id = trace_context.map_or(Uuid::nil(), |t| Uuid::from_u128(t.trace_id()));
        let query = self.parse_query::<Map>().unwrap_or_default();
        let session_id = Validation::parse_string(query.get("session_id")).or_else(|| {
            self.get_header("session-id").and_then(|header| {
                // Session IDs have the form: SID:type:realm:identifier[-thread][:count]
                header.split(':').nth(3).map(|s| s.to_string())
            })
        });
        let mut ctx = Context::new(request_id);
        ctx.set_trace_id(trace_id);
        ctx.set_session_id(session_id);
        ctx
    }

    /// Returns the trace context.
    #[inline]
    fn trace_context(&self) -> Option<TraceContext> {
        let traceparent = self.get_header("traceparent")?;
        let mut trailers = Trailers::new();
        trailers.insert("traceparent", traceparent);
        TraceContext::from_headers(&*trailers).unwrap_or(None)
    }

    /// Returns the start time.
    #[inline]
    fn start_time(&self) -> Instant {
        match self.get_context() {
            Some(ctx) => ctx.start_time(),
            None => Instant::now(),
        }
    }

    /// Returns the request ID.
    #[inline]
    fn request_id(&self) -> Uuid {
        match self.get_context() {
            Some(ctx) => ctx.request_id(),
            None => Uuid::nil(),
        }
    }

    /// Returns the trace ID.
    #[inline]
    fn trace_id(&self) -> Uuid {
        match self.get_context() {
            Some(ctx) => ctx.trace_id(),
            None => Uuid::nil(),
        }
    }

    /// Returns the session ID.
    /// See [Session Identification URI](https://www.w3.org/TR/WD-session-id).
    #[inline]
    fn session_id(&self) -> Option<&str> {
        self.get_context().and_then(|ctx| ctx.session_id())
    }

    /// Returns the request path.
    #[inline]
    fn request_path(&self) -> &str {
        self.original_uri().path()
    }

    /// Parses the route parameter by name as an instance of type `T`.
    /// The name should not include `:` or `*`.
    fn parse_param<T>(&mut self, name: &str) -> Result<T, Validation>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let path = self.matched_path();
        if path.contains([':', '*']) {
            let path_segments = path.split('/').collect::<Vec<_>>();
            if let Some(index) = path_segments
                .iter()
                .position(|segment| segment.trim_matches(|c| c == ':' || c == '*') == name)
            {
                if let Some(&param) = self
                    .request_path()
                    .splitn(path_segments.len(), '/')
                    .collect::<Vec<_>>()
                    .get(index)
                {
                    return serde_json::from_value::<T>(param.into()).map_err(|err| {
                        let mut validation = Validation::new();
                        validation.record_fail(name, param);
                        validation.record_fail("reason", err.to_string());
                        validation
                    });
                }
            }
        }

        let mut validation = Validation::new();
        validation.record_fail(name, format!("the param `{name}` does not exist"));
        Err(validation)
    }

    /// Parses the query as an instance of type `T`.
    /// Returns a default value of `T` when the query is empty.
    fn parse_query<T>(&self) -> Result<T, Validation>
    where
        T: Default + DeserializeOwned + Send + 'static,
    {
        match self.original_uri().query() {
            Some(query) => serde_qs::from_str::<T>(query).map_err(|err| {
                let mut validation = Validation::new();
                validation.record_fail("query", err.to_string());
                validation
            }),
            None => Ok(T::default()),
        }
    }

    /// Parses the request body as an instance of type `T`.
    async fn parse_body<T>(&mut self) -> Result<T, Validation>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let form_urlencoded = self
            .get_header("content-type")
            .map(|t| t.starts_with("application/x-www-form-urlencoded"))
            .unwrap_or(true);
        let body_bytes = self.to_bytes().await?;
        let result = if form_urlencoded {
            serde_urlencoded::from_bytes(body_bytes.as_ref()).map_err(|err| err.to_string())
        } else {
            serde_json::from_slice(body_bytes.as_ref()).map_err(|err| err.to_string())
        };
        result.map_err(|err| {
            let mut validation = Validation::new();
            validation.record_fail("body", err);
            validation
        })
    }

    /// Attempts to construct an instance of `Authentication` from an HTTP request.
    /// By default, the `Accept` header value is ignored and
    /// the canonicalized resource is set to the request path.
    /// You should always manually set canonicalized headers by calling
    /// `Authentication`'s method [`set_headers()`](Authentication::set_headers).
    fn parse_authentication(&self) -> Result<Authentication, Validation> {
        let method = self.request_method();
        let query = self.parse_query::<Map>().unwrap_or_default();
        let mut authentication = Authentication::new(method);
        let mut validation = Validation::new();
        if let Some(signature) = query.get("signature") {
            authentication.set_signature(signature.to_string());
            if let Some(access_key_id) = Validation::parse_string(query.get("access_key_id")) {
                authentication.set_access_key_id(access_key_id);
            } else {
                validation.record_fail("access_key_id", "must be nonempty");
            }
            if let Some(Ok(secs)) = Validation::parse_i64(query.get("expires")) {
                if DateTime::now().timestamp() <= secs {
                    let expires = DateTime::from_timestamp(secs);
                    authentication.set_expires(expires);
                } else {
                    validation.record_fail("expires", "valid period has expired");
                }
            } else {
                validation.record_fail("expires", "invalid timestamp");
            }
            if !validation.is_success() {
                return Err(validation);
            }
        } else if let Some(authorization) = self.get_header("authorization") {
            if let Some((service_name, token)) = authorization.split_once(' ') {
                authentication.set_service_name(service_name);
                if let Some((access_key_id, signature)) = token.split_once(':') {
                    authentication.set_access_key_id(access_key_id);
                    authentication.set_signature(signature.to_string());
                } else {
                    validation.record_fail("authorization", "invalid header value");
                }
            } else {
                validation.record_fail("authorization", "invalid service name");
            }
            if !validation.is_success() {
                return Err(validation);
            }
        }
        if let Some(content_md5) = self.get_header("content-md5") {
            authentication.set_content_md5(content_md5.to_string());
        }
        if let Some(date) = self.get_header("date") {
            match DateTime::parse_utc_str(date) {
                Ok(date) => {
                    let current = DateTime::now();
                    let max_tolerance = Duration::from_secs(900);
                    if date >= current - max_tolerance && date <= current + max_tolerance {
                        authentication.set_date_header("date".to_string(), date);
                    } else {
                        validation.record_fail("date", "untrusted date");
                    }
                }
                Err(err) => {
                    validation.record_fail("date", err.to_string());
                    return Err(validation);
                }
            }
        }
        authentication.set_content_type(self.get_header("content-type").map(|t| t.to_string()));
        authentication.set_resource(self.request_path().to_string(), None);
        Ok(authentication)
    }

    /// Attempts to construct an instance of `SecurityToken` from an HTTP request.
    /// The token is extracted from the `x-security-token` header.
    fn parse_security_token(&self, key: impl AsRef<[u8]>) -> Result<SecurityToken, Validation> {
        use ParseTokenError::*;
        let mut validation = Validation::new();
        if let Some(token) = self.get_header("x-security-token") {
            match SecurityToken::parse_token(key.as_ref(), token.to_string()) {
                Ok(security_token) => {
                    let query = self.parse_query::<Map>().unwrap_or_default();
                    if let Some(assignee_id) = Validation::parse_string(query.get("access_key_id"))
                    {
                        if security_token.assignee_id().as_str() != assignee_id {
                            validation.record_fail("access_key_id", "untrusted access key ID");
                        }
                    } else {
                        validation.record_fail("access_key_id", "must be nonempty");
                    }
                    if let Some(Ok(expires)) = Validation::parse_i64(query.get("expires")) {
                        if security_token.expires().timestamp() != expires {
                            validation.record_fail("expires", "untrusted timestamp");
                        }
                    } else {
                        validation.record_fail("expires", "invalid timestamp");
                    }
                    if validation.is_success() {
                        return Ok(security_token);
                    }
                }
                Err(err) => match err {
                    DecodeError(_) | InvalidFormat => {
                        validation.record_fail("x-security-token", err.to_string())
                    }
                    ParseExpiresError(_) | ValidPeriodExpired => {
                        validation.record_fail("expires", err.to_string())
                    }
                },
            }
        } else {
            validation.record_fail("x-security-token", "must be nonempty");
        }
        Err(validation)
    }

    /// Returns a `Response` or `Rejection` from an SQL query validation.
    /// The data is extracted from [`parse_query()`](RequestContext::parse_query).
    fn query_validation<S: ResponseCode>(&self, query: &mut Query) -> Result<Response<S>, Rejection>
    where
        Self: Sized,
    {
        match self.parse_query() {
            Ok(data) => {
                let validation = query.read_map(data);
                if validation.is_success() {
                    let mut res = Response::new(S::OK);
                    res.set_context(self);
                    Ok(res)
                } else {
                    Err(Rejection::BadRequest(validation))
                }
            }
            Err(validation) => Err(Rejection::BadRequest(validation)),
        }
    }

    /// Returns a `Response` or `Rejection` from a model validation.
    /// The data is extracted from [`parse_body()`](RequestContext::parse_body).
    async fn model_validation<M: Model + Send, S: ResponseCode>(
        &mut self,
        model: &mut M,
    ) -> Result<Response<S>, Rejection>
    where
        Self: Sized,
    {
        match self.parse_body().await {
            Ok(data) => {
                let validation = model.read_map(data);
                if validation.is_success() {
                    let mut res = Response::new(S::OK);
                    res.set_context(self);
                    Ok(res)
                } else {
                    Err(Rejection::BadRequest(validation))
                }
            }
            Err(validation) => Err(Rejection::BadRequest(validation)),
        }
    }

    /// Creates a new subscription instance.
    fn subscription(&self) -> Subscription {
        let mut subscription = self.parse_query::<Subscription>().unwrap_or_default();
        if subscription.session_id().is_none() {
            if let Some(session_id) = self.session_id() {
                subscription.set_session_id(session_id.to_string());
            }
        }
        subscription
    }

    /// Creates a new cloud event instance.
    fn cloud_event(&self, topic: impl Into<String>, data: impl Into<Value>) -> CloudEvent {
        let id = self.request_id().to_string();
        let source = self.request_path().to_string();
        let mut event = CloudEvent::new(id, source, topic.into(), data.into());
        if let Some(session_id) = self.session_id() {
            event.set_session_id(session_id.to_string());
        }
        event
    }
}
