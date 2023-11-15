use super::Application;
use crate::{
    error::Error,
    extension::{HeaderMapExt, JsonObjectExt, TomlTableExt},
    trace::TraceContext,
    warn, JsonValue, Map, Uuid,
};
use reqwest::{
    header::{self, HeaderMap, HeaderName},
    multipart::Form,
    Certificate, Client, Method, Request, Response, Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::{ReqwestOtelSpanBackend, TracingMiddleware};
use std::{
    borrow::Cow,
    fs,
    net::IpAddr,
    str::FromStr,
    sync::OnceLock,
    time::{Duration, Instant},
};
use task_local_extensions::Extensions;
use tracing::{field::Empty, Span};

/// Initializes the HTTP client.
pub(super) fn init<APP: Application + ?Sized>() {
    let name = APP::name();
    let version = APP::version();
    let mut client_builder = Client::builder()
        .user_agent(format!("ZinoBot/1.0 {name}/{version}"))
        .cookie_store(true)
        .gzip(true);
    let mut max_retries = 3;
    if let Some(http_client) = APP::config().get_table("http-client") {
        if let Some(timeout) = http_client.get_duration("request-timeout") {
            client_builder = client_builder.timeout(timeout);
        }
        if let Some(timeout) = http_client.get_duration("pool-idle-timeout") {
            client_builder = client_builder.pool_idle_timeout(timeout);
        }
        if let Some(max_idle_per_host) = http_client.get_usize("pool-max-idle-per-host") {
            client_builder = client_builder.pool_max_idle_per_host(max_idle_per_host);
        }
        if let Some(addr) = http_client
            .get_str("local-address")
            .and_then(|s| IpAddr::from_str(s).ok())
        {
            client_builder = client_builder.local_address(addr);
        }
        if let Some(tcp_keepalive) = http_client.get_duration("tcp-keepalive") {
            client_builder = client_builder.tcp_keepalive(tcp_keepalive);
        }
        if let Some(root_certs) = http_client.get_array("root-certs") {
            for root_cert in root_certs.iter().filter_map(|cert| cert.as_str()) {
                match fs::read(root_cert) {
                    Ok(bytes) => {
                        if root_cert.ends_with(".der") {
                            match Certificate::from_der(&bytes) {
                                Ok(cert) => {
                                    client_builder = client_builder.add_root_certificate(cert);
                                }
                                Err(err) => panic!("fail to read a DER encoded cert: {err}"),
                            }
                        } else if root_cert.ends_with(".pem") {
                            match Certificate::from_pem(&bytes) {
                                Ok(cert) => {
                                    client_builder = client_builder.add_root_certificate(cert);
                                }
                                Err(err) => panic!("fail to read a PEM encoded cert: {err}"),
                            }
                        }
                    }
                    Err(err) => panic!("fail to read cert file: {err}"),
                }
            }
        }
        if let Some(retries) = http_client.get_u32("max-retries") {
            max_retries = retries;
        }
    }

    let reqwest_client = client_builder
        .build()
        .unwrap_or_else(|err| panic!("fail to create an HTTP client: {err}"));
    SHARED_HTTP_CLIENT
        .set(reqwest_client.clone())
        .expect("fail to set an HTTP client for the application");

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(max_retries);
    let client = ClientBuilder::new(reqwest_client)
        .with(TracingMiddleware::<RequestTiming>::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    SHARED_HTTP_CLIENT_WITH_MIDDLEWARE
        .set(client)
        .expect("fail to set an HTTP client with middleware for the application");
}

/// Constructs a request builder.
pub(crate) fn request_builder(
    resource: &str,
    options: Option<&Map>,
) -> Result<RequestBuilder, Error> {
    if options.is_none() || options.is_some_and(|map| map.is_empty()) {
        let request_builder = SHARED_HTTP_CLIENT_WITH_MIDDLEWARE
            .get()
            .ok_or_else(|| warn!("fail to get the global HTTP client"))?
            .request(Method::GET, resource);
        return Ok(request_builder);
    }

    let options = options.expect("options should be nonempty");
    let method = options
        .get_str("method")
        .and_then(|s| s.parse().ok())
        .unwrap_or(Method::GET);
    let mut request_builder = SHARED_HTTP_CLIENT_WITH_MIDDLEWARE
        .get()
        .ok_or_else(|| warn!("fail to get the global HTTP client"))?
        .request(method, resource);
    let mut headers = HeaderMap::new();
    if let Some(query) = options.get("query") {
        request_builder = request_builder.query(query);
    }
    if let Some(body) = options.get("body") {
        match body {
            JsonValue::String(text) => {
                request_builder = request_builder
                    .body(text.to_owned())
                    .header(header::CONTENT_TYPE, "text/plain");
            }
            JsonValue::Object(map) => {
                let data_type = options.get_str("data_type").unwrap_or_default();
                request_builder = match data_type {
                    "form" => request_builder.form(map),
                    "json" => request_builder.json(map),
                    "multipart" => {
                        let mut form = Form::new();
                        for (key, value) in map.clone() {
                            if let JsonValue::String(value) = value {
                                form = form.text(key, value);
                            } else {
                                form = form.text(key, value.to_string());
                            }
                        }
                        request_builder.multipart(form)
                    }
                    _ => request_builder
                        .body(body.to_string())
                        .header(header::CONTENT_TYPE, "text/plain"),
                };
            }
            _ => tracing::warn!("unsupported body format"),
        }
    }
    if let Some(map) = options.get_object("headers") {
        for (key, value) in map {
            if let Ok(header_name) = HeaderName::try_from(key) {
                if let Some(header_value) = value.as_str().and_then(|s| s.parse().ok()) {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }
    if !headers.is_empty() {
        request_builder = request_builder.headers(headers);
    }
    if let Some(timeout) = options.get_u64("timeout") {
        request_builder = request_builder.timeout(Duration::from_millis(timeout));
    }
    Ok(request_builder)
}

/// Request timing.
struct RequestTiming;

impl ReqwestOtelSpanBackend for RequestTiming {
    fn on_request_start(request: &Request, extensions: &mut Extensions) -> Span {
        let method = request.method();

        // URL
        let url = request.url();
        let scheme = url.scheme();
        let host = url.host_str();
        let port = url.port();
        let path = url.path();
        let query = url.query();
        let full_url = remove_credentials(url);

        // Headers
        let headers = request.headers();
        let traceparent = headers.get_str("traceparent");
        let tracestate = headers.get_str("tracestate");
        let trace_context = traceparent.and_then(TraceContext::from_traceparent);
        let session_id = headers.get_str("session-id");
        let trace_id = trace_context
            .as_ref()
            .map(|ctx| Uuid::from_u128(ctx.trace_id()).to_string());
        let parent_id = trace_context
            .and_then(|ctx| ctx.parent_id())
            .map(|parent_id| format!("{parent_id:x}"));

        extensions.insert(Instant::now());
        if method.is_safe() {
            tracing::info_span!(
                "HTTP request",
                "otel.kind" = "client",
                "otel.name" = "zino-bot",
                "otel.status_code" = Empty,
                "url.scheme" = scheme,
                "url.path" = path,
                "url.query" = query,
                "url.full" = full_url.as_ref(),
                "http.request.method" = method.as_str(),
                "http.request.header.traceparent" = traceparent,
                "http.request.header.tracestate" = tracestate,
                "http.response.header.traceparent" = Empty,
                "http.response.header.tracestate" = Empty,
                "http.response.header.server_timing" = Empty,
                "http.response.status_code" = Empty,
                "http.client.duration" = Empty,
                "server.address" = host,
                "server.port" = port,
                "context.session_id" = session_id,
                "context.trace_id" = trace_id,
                "context.request_id" = Empty,
                "context.span_id" = Empty,
                "context.parent_id" = parent_id,
            )
        } else {
            tracing::warn_span!(
                "HTTP request",
                "otel.kind" = "client",
                "otel.name" = "zino-bot",
                "otel.status_code" = Empty,
                "url.scheme" = scheme,
                "url.path" = path,
                "url.query" = query,
                "url.full" = full_url.as_ref(),
                "http.request.method" = method.as_str(),
                "http.request.header.traceparent" = traceparent,
                "http.request.header.tracestate" = tracestate,
                "http.response.header.traceparent" = Empty,
                "http.response.header.tracestate" = Empty,
                "http.response.header.server_timing" = Empty,
                "http.response.status_code" = Empty,
                "http.client.duration" = Empty,
                "server.address" = host,
                "server.port" = port,
                "context.session_id" = session_id,
                "context.trace_id" = trace_id,
                "context.request_id" = Empty,
                "context.span_id" = Empty,
                "context.parent_id" = parent_id,
            )
        }
    }

    fn on_request_end(
        span: &Span,
        outcome: &Result<Response, reqwest_middleware::Error>,
        extensions: &mut Extensions,
    ) {
        let latency_millis = extensions
            .get::<Instant>()
            .and_then(|t| u64::try_from(t.elapsed().as_millis()).ok());
        span.record("http.client.duration", latency_millis);
        span.record(
            "context.span_id",
            span.id().map(|id| format!("{:x}", id.into_u64())),
        );
        match outcome {
            Ok(response) => {
                let headers = response.headers();
                span.record(
                    "http.response.header.traceparent",
                    headers.get_str("traceparent"),
                );
                span.record(
                    "http.response.header.tracestate",
                    headers.get_str("tracestate"),
                );
                span.record(
                    "http.response.header.server_timing",
                    headers.get_str("server-timing"),
                );
                span.record("context.request_id", headers.get_str("x-request-id"));
                span.record("http.response.status_code", response.status().as_u16());
                span.record("otel.status_code", "OK");
                tracing::info!("finished HTTP request");
            }
            Err(err) => {
                if let reqwest_middleware::Error::Reqwest(err) = err {
                    if let Some(status_code) = err.status() {
                        span.record("http.response.status_code", status_code.as_u16());
                        if status_code.is_server_error() {
                            span.record("otel.status_code", "OK");
                        } else {
                            span.record("otel.status_code", "ERROR");
                        }
                    } else {
                        span.record("otel.status_code", "ERROR");
                    }
                } else {
                    span.record("otel.status_code", "ERROR");
                }
                tracing::error!("{err}");
            }
        }
    }
}

/// Removes the username/password in the url.
fn remove_credentials(url: &Url) -> Cow<'_, str> {
    if !url.username().is_empty() || url.password().is_some() {
        let mut url = url.clone();
        url.set_username("")
            .and_then(|_| url.set_password(None))
            .ok();
        url.to_string().into()
    } else {
        url.as_ref().into()
    }
}

/// Shared HTTP client.
pub(crate) static SHARED_HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

/// Shared HTTP client with middleware.
static SHARED_HTTP_CLIENT_WITH_MIDDLEWARE: OnceLock<ClientWithMiddleware> = OnceLock::new();
