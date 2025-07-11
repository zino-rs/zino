//! Request context and validation.

use crate::{
    helper,
    response::{Rejection, Response, ResponseCode},
};
use bytes::Bytes;
use multer::Multipart;
use serde::de::DeserializeOwned;
use std::{borrow::Cow, net::IpAddr, str::FromStr, sync::Arc, time::Instant};
use zino_channel::{CloudEvent, Subscription};
use zino_core::{
    JsonValue, Map, SharedString, Uuid,
    application::Agent,
    error::Error,
    extension::HeaderMapExt,
    model::{ModelHooks, Query},
    trace::{TraceContext, TraceState},
    warn,
};
use zino_storage::NamedFile;

#[cfg(feature = "auth")]
use zino_auth::{AccessKeyId, Authentication, ParseSecurityTokenError, SecurityToken, SessionId};

#[cfg(feature = "auth")]
use zino_core::{datetime::DateTime, extension::JsonObjectExt, validation::Validation};

#[cfg(feature = "cookie")]
use cookie::{Cookie, SameSite};

#[cfg(feature = "jwt")]
use jwt_simple::algorithms::MACLike;
#[cfg(feature = "jwt")]
use zino_auth::JwtClaims;

#[cfg(any(feature = "cookie", feature = "jwt"))]
use std::time::Duration;

#[cfg(feature = "i18n")]
use fluent::FluentArgs;
#[cfg(feature = "i18n")]
use unic_langid::LanguageIdentifier;
#[cfg(feature = "i18n")]
use zino_core::i18n::{Intl, IntlError};

mod context;

pub use context::Context;

/// Request context.
pub trait RequestContext {
    /// The method type.
    type Method: AsRef<str>;
    /// The uri type.
    type Uri;

    /// Returns the request method.
    fn request_method(&self) -> &Self::Method;

    /// Returns the original request URI regardless of nesting.
    fn original_uri(&self) -> &Self::Uri;

    /// Returns the route that matches the request.
    fn matched_route(&self) -> Cow<'_, str>;

    /// Returns the request path regardless of nesting.
    fn request_path(&self) -> &str;

    /// Gets the query string of the request.
    fn get_query_string(&self) -> Option<&str>;

    /// Gets an HTTP header value with the given name.
    fn get_header(&self, name: &str) -> Option<&str>;

    /// Returns the client's remote IP.
    fn client_ip(&self) -> Option<IpAddr>;

    /// Gets the request context.
    fn get_context(&self) -> Option<Arc<Context>>;

    /// Gets the request scoped data.
    fn get_data<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Sets the request scoped data and returns the old value
    /// if an item of this type was already stored.
    fn set_data<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T>;

    /// Reads the entire request body into `Bytes`.
    async fn read_body_bytes(&mut self) -> Result<Bytes, Error>;

    /// Returns the request path segments.
    #[inline]
    fn path_segments(&self) -> Vec<&str> {
        self.request_path().trim_matches('/').split('/').collect()
    }

    /// Creates a new request context.
    fn new_context(&self) -> Context {
        // Emit metrics.
        #[cfg(feature = "metrics")]
        {
            metrics::gauge!("zino_http_requests_in_flight").increment(1.0);
            metrics::counter!(
                "zino_http_requests_total",
                "method" => self.request_method().as_ref().to_owned(),
                "route" => self.matched_route().into_owned(),
            )
            .increment(1);
        }

        // Parse tracing headers.
        let request_id = self
            .get_header("x-request-id")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(Uuid::now_v7);
        let trace_id = self
            .get_trace_context()
            .map_or_else(Uuid::now_v7, |t| Uuid::from_u128(t.trace_id()));
        let session_id = self
            .get_header("x-session-id")
            .or_else(|| self.get_header("session_id"))
            .and_then(|s| s.parse().ok());

        // Generate new context.
        let mut ctx = Context::new(request_id);
        ctx.set_instance(self.request_path().to_owned());
        ctx.set_trace_id(trace_id);
        ctx.set_session_id(session_id);

        // Set locale.
        #[cfg(feature = "i18n")]
        {
            #[cfg(feature = "cookie")]
            if let Some(cookie) = self.get_cookie("locale") {
                if let Ok(locale) = cookie.value().parse() {
                    ctx.set_locale(locale);
                    return ctx;
                }
            }

            if let Some(locale) = self
                .get_header("accept-language")
                .and_then(Intl::select_language)
            {
                ctx.set_locale(locale);
            } else {
                ctx.set_locale(Intl::default_locale().to_owned());
            }
        }
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
        trace_context.record_trace_state();
        trace_context
    }

    /// Creates a new cookie with the given name and value.
    #[cfg(feature = "cookie")]
    fn new_cookie(
        &self,
        name: SharedString,
        value: SharedString,
        max_age: Option<Duration>,
    ) -> Cookie<'static> {
        let mut cookie_builder = Cookie::build((name, value))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .path(self.request_path().to_owned());
        if let Some(max_age) = max_age.and_then(|d| d.try_into().ok()) {
            cookie_builder = cookie_builder.max_age(max_age);
        }
        cookie_builder.build()
    }

    /// Gets a cookie with the given name.
    #[cfg(feature = "cookie")]
    fn get_cookie(&self, name: &str) -> Option<Cookie<'_>> {
        self.get_header("cookie")?.split(';').find_map(|cookie| {
            if let Some((key, value)) = cookie.split_once('=') {
                (key == name).then(|| Cookie::new(key, value))
            } else {
                None
            }
        })
    }

    /// Returns the start time.
    #[inline]
    fn start_time(&self) -> Instant {
        self.get_context()
            .map(|ctx| ctx.start_time())
            .unwrap_or_else(Instant::now)
    }

    /// Returns the instance.
    #[inline]
    fn instance(&self) -> String {
        self.get_context()
            .map(|ctx| ctx.instance().to_owned())
            .unwrap_or_else(|| self.request_path().to_owned())
    }

    /// Returns the request ID.
    #[inline]
    fn request_id(&self) -> Uuid {
        self.get_context()
            .map(|ctx| ctx.request_id())
            .unwrap_or_default()
    }

    /// Returns the trace ID.
    #[inline]
    fn trace_id(&self) -> Uuid {
        self.get_context()
            .map(|ctx| ctx.trace_id())
            .unwrap_or_default()
    }

    /// Returns the session ID.
    #[inline]
    fn session_id(&self) -> Option<String> {
        self.get_context()
            .and_then(|ctx| ctx.session_id().map(|s| s.to_owned()))
    }

    /// Returns the locale.
    #[cfg(feature = "i18n")]
    #[inline]
    fn locale(&self) -> Option<LanguageIdentifier> {
        self.get_context().and_then(|ctx| ctx.locale().cloned())
    }

    /// Gets the data type by parsing the `content-type` header.
    ///
    /// # Note
    ///
    /// Currently, we support the following values: `bytes` | `csv` | `form` | `json` | `multipart`
    /// | `ndjson` | `text`.
    fn data_type(&self) -> Option<&str> {
        self.get_header("content-type")
            .map(|content_type| {
                if let Some((essence, _)) = content_type.split_once(';') {
                    essence
                } else {
                    content_type
                }
            })
            .map(helper::get_data_type)
    }

    /// Gets the route parameter by name.
    /// The name should not include `:`, `*`, `{` or `}`.
    ///
    /// # Note
    ///
    /// Please note that it does not handle the percent-decoding.
    /// You can use [`decode_param()`](Self::decode_param) or [`parse_param()`](Self::parse_param)
    /// if you need percent-decoding.
    fn get_param(&self, name: &str) -> Option<&str> {
        const CAPTURES: [char; 4] = [':', '*', '{', '}'];
        if let Some(index) = self
            .matched_route()
            .split('/')
            .position(|segment| segment.trim_matches(CAPTURES.as_slice()) == name)
        {
            self.request_path().splitn(index + 2, '/').nth(index)
        } else {
            None
        }
    }

    /// Decodes the UTF-8 percent-encoded route parameter by name.
    fn decode_param(&self, name: &str) -> Result<Cow<'_, str>, Rejection> {
        if let Some(value) = self.get_param(name) {
            percent_encoding::percent_decode_str(value)
                .decode_utf8()
                .map_err(|err| Rejection::from_validation_entry(name.to_owned(), err).context(self))
        } else {
            Err(Rejection::from_validation_entry(
                name.to_owned(),
                warn!("param `{}` does not exist", name),
            )
            .context(self))
        }
    }

    /// Parses the route parameter by name as an instance of type `T`.
    /// The name should not include `:`, `*`, `{` or `}`.
    fn parse_param<T: FromStr<Err: Into<Error>>>(&self, name: &str) -> Result<T, Rejection> {
        if let Some(param) = self.get_param(name) {
            percent_encoding::percent_decode_str(param)
                .decode_utf8_lossy()
                .parse::<T>()
                .map_err(|err| Rejection::from_validation_entry(name.to_owned(), err).context(self))
        } else {
            Err(Rejection::from_validation_entry(
                name.to_owned(),
                warn!("param `{}` does not exist", name),
            )
            .context(self))
        }
    }

    /// Gets the query value of the URI by name.
    ///
    /// # Note
    ///
    /// Please note that it does not handle the percent-decoding.
    /// You can use [`decode_query()`](Self::decode_query) or [`parse_query()`](Self::parse_query)
    /// if you need percent-decoding.
    fn get_query(&self, name: &str) -> Option<&str> {
        self.get_query_string()?.split('&').find_map(|param| {
            if let Some((key, value)) = param.split_once('=') {
                (key == name).then_some(value)
            } else {
                None
            }
        })
    }

    /// Decodes the UTF-8 percent-encoded query value of the URI by name.
    fn decode_query(&self, name: &str) -> Result<Cow<'_, str>, Rejection> {
        if let Some(value) = self.get_query(name) {
            percent_encoding::percent_decode_str(value)
                .decode_utf8()
                .map_err(|err| Rejection::from_validation_entry(name.to_owned(), err).context(self))
        } else {
            Err(Rejection::from_validation_entry(
                name.to_owned(),
                warn!("query value `{}` does not exist", name),
            )
            .context(self))
        }
    }

    /// Parses the query as an instance of type `T`.
    /// Returns a default value of `T` when the query is empty.
    /// If the query has a `timestamp` parameter, it will be used to prevent replay attacks.
    fn parse_query<T: Default + DeserializeOwned>(&self) -> Result<T, Rejection> {
        if let Some(query) = self.get_query_string() {
            #[cfg(feature = "jwt")]
            if let Some(timestamp) = self.get_query("timestamp").and_then(|s| s.parse().ok()) {
                let duration = DateTime::from_timestamp(timestamp).span_between_now();
                if duration > zino_auth::default_time_tolerance() {
                    let err = warn!("timestamp `{}` can not be trusted", timestamp);
                    let rejection = Rejection::from_validation_entry("timestamp", err);
                    return Err(rejection.context(self));
                }
            }
            serde_qs::from_str::<T>(query)
                .map_err(|err| Rejection::from_validation_entry("query", err).context(self))
        } else {
            Ok(T::default())
        }
    }

    /// Parses the request body as an instance of type `T`.
    ///
    /// # Note
    ///
    /// Currently, we have built-in support for the following `content-type` header values:
    ///
    /// - `application/json`
    /// - `application/problem+json`
    /// - `application/x-www-form-urlencoded`
    async fn parse_body<T: DeserializeOwned>(&mut self) -> Result<T, Rejection> {
        let data_type = self.data_type().unwrap_or("form");
        if data_type.contains('/') {
            let err = warn!(
                "deserialization of the data type `{}` is unsupported",
                data_type
            );
            let rejection = Rejection::from_validation_entry("data_type", err).context(self);
            return Err(rejection);
        }

        let is_form = data_type == "form";
        let bytes = self
            .read_body_bytes()
            .await
            .map_err(|err| Rejection::from_validation_entry("body", err).context(self))?;
        if is_form {
            serde_qs::from_bytes(&bytes)
                .map_err(|err| Rejection::from_validation_entry("body", err).context(self))
        } else {
            serde_json::from_slice(&bytes)
                .map_err(|err| Rejection::from_validation_entry("body", err).context(self))
        }
    }

    /// Parses the request body as a multipart, which is commonly used with file uploads.
    async fn parse_multipart(&mut self) -> Result<Multipart<'_>, Rejection> {
        let Some(content_type) = self.get_header("content-type") else {
            return Err(Rejection::from_validation_entry(
                "content_type",
                warn!("invalid `content-type` header"),
            )
            .context(self));
        };
        match multer::parse_boundary(content_type) {
            Ok(boundary) => {
                let result = self.read_body_bytes().await.map_err(|err| err.to_string());
                let stream = futures::stream::once(async { result });
                Ok(Multipart::new(stream, boundary))
            }
            Err(err) => Err(Rejection::from_validation_entry("boundary", err).context(self)),
        }
    }

    /// Parses the request body as a file.
    async fn parse_file(&mut self) -> Result<NamedFile, Rejection> {
        let multipart = self.parse_multipart().await?;
        NamedFile::try_from_multipart(multipart)
            .await
            .map_err(|err| Rejection::from_validation_entry("body", err).context(self))
    }

    /// Parses the request body as a list of files.
    async fn parse_files(&mut self) -> Result<Vec<NamedFile>, Rejection> {
        let multipart = self.parse_multipart().await?;
        NamedFile::try_collect_from_multipart(multipart)
            .await
            .map_err(|err| Rejection::from_validation_entry("body", err).context(self))
    }

    /// Parses the multipart form as an instance of `T` with the `name` and a list of files.
    async fn parse_form<T: DeserializeOwned>(
        &mut self,
        name: &str,
    ) -> Result<(Option<T>, Vec<NamedFile>), Rejection> {
        let multipart = self.parse_multipart().await?;
        helper::parse_form(multipart, name)
            .await
            .map_err(|err| Rejection::from_validation_entry("body", err).context(self))
    }

    /// Parses the `multipart/form-data` as an instance of type `T` and a list of files.
    async fn parse_form_data<T: DeserializeOwned>(
        &mut self,
    ) -> Result<(T, Vec<NamedFile>), Rejection> {
        let multipart = self.parse_multipart().await?;
        helper::parse_form_data(multipart)
            .await
            .map_err(|err| Rejection::from_validation_entry("body", err).context(self))
    }

    /// Attempts to construct an instance of `Authentication` from an HTTP request.
    /// The value is extracted from the query or the `authorization` header.
    /// By default, the `Accept` header value is ignored and
    /// the canonicalized resource is set to the request path.
    #[cfg(feature = "auth")]
    fn parse_authentication(&self) -> Result<Authentication, Rejection> {
        let method = self.request_method();
        let query = self.parse_query::<Map>().unwrap_or_default();
        let mut authentication = Authentication::new(method.as_ref());
        let mut validation = Validation::new();
        if let Some(signature) = query.get_str("signature") {
            authentication.set_signature(signature.to_owned());
            if let Some(access_key_id) = query.parse_string("access_key_id") {
                authentication.set_access_key_id(access_key_id);
            } else {
                validation.record("access_key_id", "should be nonempty");
            }
            if let Some(Ok(secs)) = query.parse_i64("expires") {
                if DateTime::now().timestamp() <= secs {
                    let expires = DateTime::from_timestamp(secs);
                    authentication.set_expires(Some(expires));
                } else {
                    validation.record("expires", "valid period has expired");
                }
            } else {
                validation.record("expires", "invalid timestamp");
            }
            if !validation.is_success() {
                return Err(Rejection::bad_request(validation).context(self));
            }
        } else if let Some(authorization) = self.get_header("authorization") {
            if let Some((service_name, token)) = authorization.split_once(' ') {
                authentication.set_service_name(service_name);
                if let Some((access_key_id, signature)) = token.split_once(':') {
                    authentication.set_access_key_id(access_key_id);
                    authentication.set_signature(signature.to_owned());
                } else {
                    validation.record("authorization", "invalid header value");
                }
            } else {
                validation.record("authorization", "invalid service name");
            }
            if !validation.is_success() {
                return Err(Rejection::bad_request(validation).context(self));
            }
        }
        if let Some(content_md5) = self.get_header("content-md5") {
            authentication.set_content_md5(content_md5.to_owned());
        }
        if let Some(date) = self.get_header("date") {
            match DateTime::parse_utc_str(date) {
                Ok(date) => {
                    #[cfg(feature = "jwt")]
                    if date.span_between_now() <= zino_auth::default_time_tolerance() {
                        authentication.set_date_header("date", date);
                    } else {
                        validation.record("date", "untrusted date");
                    }
                    #[cfg(not(feature = "jwt"))]
                    authentication.set_date_header("date", date);
                }
                Err(err) => {
                    validation.record_fail("date", err);
                    return Err(Rejection::bad_request(validation).context(self));
                }
            }
        }
        authentication.set_content_type(self.get_header("content-type").map(|s| s.to_owned()));
        authentication.set_resource(self.request_path().to_owned(), None);
        Ok(authentication)
    }

    /// Attempts to construct an instance of `AccessKeyId` from an HTTP request.
    /// The value is extracted from the query parameter `access_key_id`
    /// or the `authorization` header.
    #[cfg(feature = "auth")]
    fn parse_access_key_id(&self) -> Result<AccessKeyId, Rejection> {
        if let Some(access_key_id) = self.get_query("access_key_id") {
            Ok(access_key_id.into())
        } else {
            let mut validation = Validation::new();
            if let Some(authorization) = self.get_header("authorization") {
                if let Some((_, token)) = authorization.split_once(' ') {
                    let access_key_id = if let Some((access_key_id, _)) = token.split_once(':') {
                        access_key_id
                    } else {
                        token
                    };
                    return Ok(access_key_id.into());
                } else {
                    validation.record("authorization", "invalid service name");
                }
            } else {
                validation.record("authorization", "invalid value to get the access key id");
            }
            Err(Rejection::bad_request(validation).context(self))
        }
    }

    /// Attempts to construct an instance of `SecurityToken` from an HTTP request.
    /// The value is extracted from the `x-security-token` header.
    #[cfg(feature = "auth")]
    fn parse_security_token(&self, key: &[u8]) -> Result<SecurityToken, Rejection> {
        use ParseSecurityTokenError::*;
        let query = self.parse_query::<Map>()?;
        let mut validation = Validation::new();
        if let Some(token) = self
            .get_header("x-security-token")
            .or_else(|| query.get_str("security_token"))
        {
            match SecurityToken::parse_with(token.to_owned(), key) {
                Ok(security_token) => {
                    if let Some(access_key_id) = query.get_str("access_key_id") {
                        if security_token.access_key_id().as_str() != access_key_id {
                            validation.record("access_key_id", "untrusted access key ID");
                        }
                    }
                    if let Some(Ok(expires)) = query.parse_i64("expires") {
                        if security_token.expires_at().timestamp() != expires {
                            validation.record("expires", "untrusted timestamp");
                        }
                    }
                    if validation.is_success() {
                        return Ok(security_token);
                    }
                }
                Err(err) => {
                    let field = match err {
                        DecodeError(_) | InvalidFormat => "security_token",
                        ParseExpiresError(_) | ValidPeriodExpired(_) => "expires",
                    };
                    validation.record_fail(field, err);
                }
            }
        } else {
            validation.record("security_token", "should be nonempty");
        }
        Err(Rejection::bad_request(validation).context(self))
    }

    /// Attempts to construct an instance of `SessionId` from an HTTP request.
    /// The value is extracted from the `x-session-id` or `session-id` header.
    #[cfg(feature = "auth")]
    fn parse_session_id(&self) -> Result<SessionId, Rejection> {
        self.get_header("x-session-id")
            .or_else(|| self.get_header("session-id"))
            .ok_or_else(|| {
                Rejection::from_validation_entry(
                    "session_id",
                    warn!("a `session-id` or `x-session-id` header is required"),
                )
                .context(self)
            })
            .and_then(|session_id| {
                SessionId::parse(session_id).map_err(|err| {
                    Rejection::from_validation_entry("session_id", err).context(self)
                })
            })
    }

    /// Attempts to construct an instance of `JwtClaims` from an HTTP request.
    /// The value is extracted from the query parameter `access_token` or
    /// the `authorization` header.
    #[cfg(feature = "jwt")]
    fn parse_jwt_claims<T, K>(&self, key: &K) -> Result<JwtClaims<T>, Rejection>
    where
        T: Default + serde::Serialize + DeserializeOwned,
        K: MACLike,
    {
        let (param, mut token) = match self.get_query("access_token") {
            Some(access_token) => ("access_token", access_token),
            None => ("authorization", ""),
        };
        if let Some(authorization) = self.get_header("authorization") {
            token = authorization
                .strip_prefix("Bearer ")
                .unwrap_or(authorization);
        }
        if token.is_empty() {
            let mut validation = Validation::new();
            validation.record(param, "JWT token is absent");
            return Err(Rejection::bad_request(validation).context(self));
        }

        let mut options = zino_auth::default_verification_options();
        options.reject_before = self
            .get_query("timestamp")
            .and_then(|s| s.parse().ok())
            .map(|i| Duration::from_secs(i).into());
        options.required_nonce = self.get_query("nonce").map(|s| s.to_owned());

        match key.verify_token(token, Some(options)) {
            Ok(claims) => Ok(claims.into()),
            Err(err) => {
                let message = format!("401 Unauthorized: {err}");
                Err(Rejection::with_message(message).context(self))
            }
        }
    }

    /// Returns a `Response` or `Rejection` from a model query validation.
    /// The data is extracted from [`parse_query()`](RequestContext::parse_query).
    fn query_validation<S>(&self, query: &mut Query) -> Result<Response<S>, Rejection>
    where
        Self: Sized,
        S: ResponseCode,
    {
        match self.parse_query() {
            Ok(data) => {
                let validation = query.read_map(&data);
                if validation.is_success() {
                    Ok(Response::with_context(S::OK, self))
                } else {
                    Err(Rejection::bad_request(validation).context(self))
                }
            }
            Err(rejection) => Err(rejection),
        }
    }

    /// Returns a `Response` or `Rejection` from a model validation.
    /// The data is extracted from [`parse_body()`](RequestContext::parse_body).
    async fn model_validation<M, S>(&mut self, model: &mut M) -> Result<Response<S>, Rejection>
    where
        Self: Sized,
        M: ModelHooks,
        S: ResponseCode,
    {
        let data_type = self.data_type().unwrap_or("form");
        if data_type.contains('/') {
            let err = warn!(
                "deserialization of the data type `{}` is unsupported",
                data_type
            );
            let rejection = Rejection::from_validation_entry("data_type", err).context(self);
            return Err(rejection);
        }
        M::before_extract()
            .await
            .map_err(|err| Rejection::from_error(err).context(self))?;

        let is_form = data_type == "form";
        let bytes = self
            .read_body_bytes()
            .await
            .map_err(|err| Rejection::from_validation_entry("body", err).context(self))?;
        let extension = self.get_data::<M::Extension>();
        if is_form {
            let mut data = serde_qs::from_bytes(&bytes)
                .map_err(|err| Rejection::from_validation_entry("body", err).context(self))?;
            match M::before_validation(&mut data, extension.as_ref()).await {
                Ok(()) => {
                    let validation = model.read_map(&data);
                    model
                        .after_validation(&mut data)
                        .await
                        .map_err(|err| Rejection::from_error(err).context(self))?;
                    if let Some(extension) = extension {
                        model
                            .after_extract(extension)
                            .await
                            .map_err(|err| Rejection::from_error(err).context(self))?;
                    }
                    if validation.is_success() {
                        Ok(Response::with_context(S::OK, self))
                    } else {
                        Err(Rejection::bad_request(validation).context(self))
                    }
                }
                Err(err) => Err(Rejection::from_error(err).context(self)),
            }
        } else {
            let mut data = serde_json::from_slice(&bytes)
                .map_err(|err| Rejection::from_validation_entry("body", err).context(self))?;
            match M::before_validation(&mut data, extension.as_ref()).await {
                Ok(()) => {
                    let validation = model.read_map(&data);
                    model
                        .after_validation(&mut data)
                        .await
                        .map_err(|err| Rejection::from_error(err).context(self))?;
                    if let Some(extension) = extension {
                        model
                            .after_extract(extension)
                            .await
                            .map_err(|err| Rejection::from_error(err).context(self))?;
                    }
                    if validation.is_success() {
                        Ok(Response::with_context(S::OK, self))
                    } else {
                        Err(Rejection::bad_request(validation).context(self))
                    }
                }
                Err(err) => Err(Rejection::from_error(err).context(self)),
            }
        }
    }

    /// Makes an HTTP request to the provided URL.
    async fn fetch(&self, url: &str, options: Option<&Map>) -> Result<reqwest::Response, Error> {
        let trace_context = self.new_trace_context();
        Agent::request_builder(url, options)?
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await
            .map_err(Error::from)
    }

    /// Makes an HTTP request to the provided URL and
    /// deserializes the response body via JSON.
    async fn fetch_json<T: DeserializeOwned>(
        &self,
        url: &str,
        options: Option<&Map>,
    ) -> Result<T, Error> {
        let response = self.fetch(url, options).await?.error_for_status()?;
        let data = if response.headers().has_json_content_type() {
            response.json().await?
        } else {
            let text = response.text().await?;
            serde_json::from_str(&text)?
        };
        Ok(data)
    }

    /// Translates the localization message.
    #[cfg(feature = "i18n")]
    fn translate(
        &self,
        message: &str,
        args: Option<FluentArgs>,
    ) -> Result<SharedString, IntlError> {
        if let Some(locale) = self.locale() {
            Intl::translate_with(message, args, &locale)
        } else {
            Intl::translate(message, args)
        }
    }

    /// Constructs a new subscription instance.
    fn subscription(&self) -> Subscription {
        let mut subscription = self.parse_query::<Subscription>().unwrap_or_default();
        if subscription.session_id().is_none() {
            if let Some(session_id) = self.session_id() {
                subscription.set_session_id(Some(session_id));
            }
        }
        subscription
    }

    /// Constructs a new cloud event instance.
    fn cloud_event(&self, event_type: SharedString, data: JsonValue) -> CloudEvent {
        let id = self.request_id();
        let source = self.instance();
        let mut event = CloudEvent::new(id, source, event_type);
        if let Some(session_id) = self.session_id() {
            event.set_session_id(session_id);
        }
        event.set_data(data);
        event
    }
}
