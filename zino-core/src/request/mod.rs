//! Request context and validation.

use crate::{
    application::http_client,
    authentication::{Authentication, ParseSecurityTokenError, SecurityToken, SessionId},
    channel::{CloudEvent, Subscription},
    database::{Model, Query},
    datetime::DateTime,
    i18n,
    response::{Rejection, Response, ResponseCode},
    trace::{TraceContext, TraceState},
    BoxError, Map, SharedString, Uuid,
};
use fluent::FluentArgs;
use http::uri::Uri;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::{Duration, Instant};
use toml::value::Table;
use unic_langid::{langid, LanguageIdentifier};

mod context;
mod validation;

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

    /// Returns the route that matches the request.
    fn matched_route(&self) -> &str;

    /// Returns the original request URI regardless of nesting.
    fn original_uri(&self) -> &Uri;

    /// Parses the request body as an instance of type `T`.
    async fn parse_body<T>(&mut self) -> Result<T, Validation>
    where
        T: DeserializeOwned + Send + 'static;

    /// Attempts to send a message.
    fn try_send(&self, message: impl Into<CloudEvent>) -> Result<(), Rejection>;

    /// Creates a new request context.
    fn new_context(&self) -> Context {
        // Emit metrics.
        metrics::increment_gauge!("zino_http_requests_pending", 1.0);
        metrics::increment_counter!(
            "zino_http_requests_total",
            "method" => self.request_method().to_owned(),
            "route" => self.matched_route().to_owned(),
        );

        // Parse tracing headers.
        let request_id = self
            .get_header("x-request-id")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(Uuid::new_v4);
        let trace_id = self
            .get_trace_context()
            .map_or_else(Uuid::new_v4, |t| Uuid::from_u128(t.trace_id()));
        let session_id = self.get_header("session-id").and_then(|s| s.parse().ok());

        // Parse locale.
        let locale = self
            .get_header("accept-language")
            .map(|s| {
                if let Some((locale, _)) = s.split_once(',') {
                    locale
                } else {
                    s
                }
            })
            .unwrap_or("en-US");

        // Generate new context.
        let mut ctx = Context::new(request_id);
        ctx.set_trace_id(trace_id);
        ctx.set_session_id(session_id);
        ctx.set_locale(locale);
        ctx
    }

    /// Returns the trace context by parsing the `traceparent` and `tracestate` header values.
    #[inline]
    fn get_trace_context(&self) -> Option<TraceContext> {
        let traceparent = self.get_header("traceparent")?;
        let mut trace_context = TraceContext::from_traceparent(traceparent)?;
        if let Some(tracestate) = self.get_header("tracestate") {
            *trace_context.trace_state_mut() = TraceState::from_tracestate(tracestate);
        }
        Some(trace_context)
    }

    /// Creates a new `TraceContext`.
    fn new_trace_context(&self) -> TraceContext {
        let mut trace_context = self
            .get_trace_context()
            .or_else(|| {
                self.get_context()
                    .map(|ctx| TraceContext::with_trace_id(ctx.trace_id()))
            })
            .map(|t| t.child())
            .unwrap_or_default();
        let span_id = trace_context.span_id();
        trace_context
            .trace_state_mut()
            .push("zino", format!("{span_id:x}"));
        trace_context
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
    #[inline]
    fn session_id(&self) -> Option<&str> {
        self.get_context().and_then(|ctx| ctx.session_id())
    }

    /// Returns the locale.
    #[inline]
    fn locale(&self) -> Option<&LanguageIdentifier> {
        self.get_context().and_then(|ctx| ctx.locale())
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
        const CAPTURES: [char; 2] = [':', '*'];
        let route = self.matched_route();
        if route.contains(CAPTURES) {
            let segments = route.split('/').collect::<Vec<_>>();
            if let Some(index) = segments
                .iter()
                .position(|segment| segment.trim_matches(CAPTURES.as_slice()) == name)
            {
                let path = self.request_path();
                if let Some(&param) = path
                    .splitn(segments.len(), '/')
                    .collect::<Vec<_>>()
                    .get(index)
                {
                    return serde_json::from_value::<T>(param.into()).map_err(|err| {
                        let mut validation = Validation::new();
                        validation.record_fail(name.to_owned(), err);
                        validation
                    });
                }
            }
        }

        let mut validation = Validation::new();
        validation.record_fail(
            name.to_owned(),
            format!("the param `{name}` does not exist"),
        );
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
                validation.record_fail("query", err);
                validation
            }),
            None => Ok(T::default()),
        }
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
        if let Some(signature) = query.get("signature").and_then(|v| v.as_str()) {
            authentication.set_signature(signature.to_owned());
            if let Some(access_key_id) = Validation::parse_string(query.get("access_key_id")) {
                authentication.set_access_key_id(access_key_id);
            } else {
                validation.record_fail("access_key_id", "should be nonempty");
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
                    authentication.set_signature(signature.to_owned());
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
            authentication.set_content_md5(content_md5.to_owned());
        }
        if let Some(date) = self.get_header("date") {
            match DateTime::parse_utc_str(date) {
                Ok(date) => {
                    let current = DateTime::now();
                    let max_tolerance = Duration::from_secs(900);
                    if date >= current - max_tolerance && date <= current + max_tolerance {
                        authentication.set_date_header("date", date);
                    } else {
                        validation.record_fail("date", "untrusted date");
                    }
                }
                Err(err) => {
                    validation.record_fail("date", err);
                    return Err(validation);
                }
            }
        }
        authentication.set_content_type(self.get_header("content-type").map(|s| s.to_owned()));
        authentication.set_resource(self.request_path().to_owned(), None);
        Ok(authentication)
    }

    /// Attempts to construct an instance of `SecurityToken` from an HTTP request.
    /// The value is extracted from the `x-security-token` header.
    fn parse_security_token(&self, key: impl AsRef<[u8]>) -> Result<SecurityToken, Validation> {
        use ParseSecurityTokenError::*;
        let mut validation = Validation::new();
        if let Some(token) = self.get_header("x-security-token") {
            match SecurityToken::parse_with(token.to_owned(), key.as_ref()) {
                Ok(security_token) => {
                    let query = self.parse_query::<Map>().unwrap_or_default();
                    if let Some(assignee_id) = Validation::parse_string(query.get("access_key_id"))
                    {
                        if security_token.assignee_id().as_str() != assignee_id {
                            validation.record_fail("access_key_id", "untrusted access key ID");
                        }
                    } else {
                        validation.record_fail("access_key_id", "should be nonempty");
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
                Err(err) => {
                    let field = match err {
                        DecodeError(_) | InvalidFormat => "x-security-token",
                        ParseExpiresError(_) | ValidPeriodExpired => "expires",
                    };
                    validation.record_fail(field, err.to_string());
                }
            }
        } else {
            validation.record_fail("x-security-token", "should be nonempty");
        }
        Err(validation)
    }

    /// Attempts to construct an instance of `SessionId` from an HTTP request.
    /// The value is extracted from the `session-id` header.
    fn parse_session_id(&self) -> Result<SessionId, Validation> {
        let mut validation = Validation::new();
        if let Some(session_id) = self.get_header("session-id") {
            match SessionId::parse(session_id) {
                Ok(session_id) => return Ok(session_id),
                Err(err) => {
                    validation.record_fail("session-id", err);
                }
            }
        } else {
            validation.record_fail("session-id", "should be nonempty");
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
                    let res = Response::with_context(S::OK, self);
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
                    let res = Response::with_context(S::OK, self);
                    Ok(res)
                } else {
                    Err(Rejection::BadRequest(validation))
                }
            }
            Err(validation) => Err(Rejection::BadRequest(validation)),
        }
    }

    /// Makes an HTTP request to the provided resource
    /// using [`reqwest`](https://crates.io/crates/reqwest).
    async fn fetch(
        &self,
        resource: impl IntoUrl,
        options: impl Into<Option<Map>>,
    ) -> Result<reqwest::Response, BoxError> {
        let trace_context = self.new_trace_context();
        http_client::request_builder(resource, options)?
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await
            .map_err(BoxError::from)
    }

    /// Translates the localization message.
    fn translate<'a>(
        &self,
        message: &str,
        args: impl Into<Option<FluentArgs<'a>>>,
    ) -> Result<SharedString, BoxError> {
        let args = args.into();
        match self.locale() {
            Some(locale) => i18n::translate(locale, message, args),
            None => i18n::translate(&langid!("en-US"), message, args),
        }
    }

    /// Creates a new subscription instance.
    fn subscription(&self) -> Subscription {
        let mut subscription = self.parse_query::<Subscription>().unwrap_or_default();
        if subscription.session_id().is_none() {
            if let Some(session_id) = self.session_id() {
                subscription.set_session_id(Some(session_id.to_owned()));
            }
        }
        subscription
    }

    /// Creates a new cloud event instance.
    fn cloud_event(&self, topic: impl Into<String>, data: impl Into<Value>) -> CloudEvent {
        let id = self.request_id().to_string();
        let source = self.request_path().to_owned();
        let mut event = CloudEvent::new(id, source, topic.into(), data.into());
        if let Some(session_id) = self.session_id() {
            event.set_session_id(session_id.to_owned());
        }
        event
    }
}
