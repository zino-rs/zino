use axum::{
    body::{Body, BoxBody, Bytes},
    http::{HeaderMap, Request, Response},
};
use std::{sync::LazyLock, time::Duration};
use tower_http::{
    classify::{SharedClassifier, StatusInRangeAsFailures, StatusInRangeFailureClass},
    trace::TraceLayer,
};
use tracing::{field::Empty, Span};
use zino_core::{application::Application, extension::HeaderMapExt, trace::TraceContext, Uuid};

/// Type aliases.
type CustomMakeSpan = fn(&Request<Body>) -> Span;
type CustomOnRequest = fn(&Request<Body>, &Span);
type CustomOnResponse = fn(&Response<BoxBody>, Duration, &Span);
type CustomOnBodyChunk = fn(&Bytes, Duration, &Span);
type CustomOnEos = fn(Option<&HeaderMap>, Duration, &Span);
type CustomOnFailure = fn(StatusInRangeFailureClass, Duration, &Span);
type CustomTraceLayer = TraceLayer<
    SharedClassifier<StatusInRangeAsFailures>,
    CustomMakeSpan,
    CustomOnRequest,
    CustomOnResponse,
    CustomOnBodyChunk,
    CustomOnEos,
    CustomOnFailure,
>;

/// Tracing middleware.
pub(crate) static TRACING_MIDDLEWARE: LazyLock<CustomTraceLayer> = LazyLock::new(|| {
    let classifier = StatusInRangeAsFailures::new_for_client_and_server_errors();
    TraceLayer::new(classifier.into_make_classifier())
        .make_span_with(custom_make_span as CustomMakeSpan)
        .on_request(custom_on_request as CustomOnRequest)
        .on_response(custom_on_response as CustomOnResponse)
        .on_body_chunk(custom_on_body_chunk as CustomOnBodyChunk)
        .on_eos(custom_on_eos as CustomOnEos)
        .on_failure(custom_on_failure as CustomOnFailure)
});

fn custom_make_span(request: &Request<Body>) -> Span {
    let name = crate::Cluster::name();
    let method = request.method();

    // URI
    let uri = request.uri();
    let scheme = uri.scheme_str();
    let host = uri.host();
    let port = uri.port_u16();
    let path = uri.path();
    let query = uri.query();

    // Headers
    let headers = request.headers();
    let client_ip = headers.get_client_ip().map(|ip| ip.to_string());
    let user_agent = headers.get_str("user-agent");

    if method.is_safe() {
        tracing::info_span!(
            "HTTP request",
            "otel.kind" = "server",
            "otel.name" = name,
            "otel.status_code" = Empty,
            "url.scheme" = scheme,
            "url.path" = path,
            "url.query" = query,
            "http.request.method" = method.as_str(),
            "http.request.header.traceparent" = Empty,
            "http.request.header.tracestate" = Empty,
            "http.response.header.traceparent" = Empty,
            "http.response.header.tracestate" = Empty,
            "http.response.header.server_timing" = Empty,
            "http.response.status_code" = Empty,
            "client.address" = client_ip,
            "server.address" = host,
            "server.port" = port,
            "user_agent.original" = user_agent,
            "context.session_id" = Empty,
            "context.trace_id" = Empty,
            "context.request_id" = Empty,
            "context.span_id" = Empty,
            "context.parent_id" = Empty,
        )
    } else {
        tracing::warn_span!(
            "HTTP request",
            "otel.kind" = "server",
            "otel.name" = name,
            "otel.status_code" = Empty,
            "url.scheme" = scheme,
            "url.path" = path,
            "url.query" = query,
            "http.request.method" = method.as_str(),
            "http.request.header.traceparent" = Empty,
            "http.request.header.tracestate" = Empty,
            "http.response.header.traceparent" = Empty,
            "http.response.header.tracestate" = Empty,
            "http.response.header.server_timing" = Empty,
            "http.response.status_code" = Empty,
            "client.address" = client_ip,
            "server.address" = host,
            "server.port" = port,
            "user_agent.original" = user_agent,
            "context.session_id" = Empty,
            "context.trace_id" = Empty,
            "context.request_id" = Empty,
            "context.span_id" = Empty,
            "context.parent_id" = Empty,
        )
    }
}

fn custom_on_request(request: &Request<Body>, span: &Span) {
    let headers = request.headers();
    let traceparent = headers.get_str("traceparent");
    let trace_context = traceparent.and_then(TraceContext::from_traceparent);
    span.record("http.request.header.traceparent", traceparent);
    span.record(
        "http.request.header.tracestate",
        headers.get_str("tracestate"),
    );
    span.record(
        "context.parent_id",
        trace_context
            .and_then(|ctx| ctx.parent_id())
            .map(|parent_id| format!("{parent_id:x}")),
    );
    span.record("context.session_id", headers.get_str("session-id"));
    span.record(
        "context.span_id",
        span.id().map(|id| format!("{:x}", id.into_u64())),
    );
    tracing::debug!("started processing request");
}

fn custom_on_response(response: &Response<BoxBody>, latency: Duration, span: &Span) {
    let headers = response.headers();
    let traceparent = headers.get_str("traceparent");
    span.record("http.response.header.traceparent", traceparent);
    span.record(
        "http.response.header.tracestate",
        headers.get_str("tracestate"),
    );
    span.record(
        "http.response.header.server_timing",
        headers.get_str("server-timing"),
    );
    span.record(
        "context.trace_id",
        traceparent
            .and_then(TraceContext::from_traceparent)
            .map(|ctx| Uuid::from_u128(ctx.trace_id()).to_string()),
    );
    span.record("context.request_id", headers.get_str("x-request-id"));
    span.record("http.response.status_code", response.status().as_u16());
    span.record(
        "http.server.duration",
        u64::try_from(latency.as_millis()).ok(),
    );
    span.record("otel.status_code", "OK");
    tracing::info!("finished processing request");
}

fn custom_on_body_chunk(chunk: &Bytes, _latency: Duration, _span: &Span) {
    tracing::debug!("flushed {} bytes", chunk.len());
}

fn custom_on_eos(_trailers: Option<&HeaderMap>, stream_duration: Duration, span: &Span) {
    span.record("otel.status_code", "OK");
    tracing::debug!(
        stream_duration = u64::try_from(stream_duration.as_millis()).ok(),
        "end of stream",
    );
}

fn custom_on_failure(error: StatusInRangeFailureClass, latency: Duration, span: &Span) {
    match error {
        StatusInRangeFailureClass::StatusCode(status_code) => {
            span.record("http.response.status_code", status_code.as_u16());
            if status_code.is_client_error() {
                span.record("otel.status_code", "OK");
                tracing::warn!("response failed");
            } else {
                span.record("otel.status_code", "ERROR");
                tracing::error!("response failed");
            }
        }
        StatusInRangeFailureClass::Error(err) => {
            span.record("otel.status_code", "ERROR");
            tracing::error!(err);
        }
    }
}
