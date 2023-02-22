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
use zino_core::{application::Application, extend::HeaderMapExt, trace::TraceContext, Uuid};

// Type aliases.
type NewMakeSpan = fn(&Request<Body>) -> Span;
type NewOnRequest = fn(&Request<Body>, &Span);
type NewOnResponse = fn(&Response<BoxBody>, Duration, &Span);
type NewOnBodyChunk = fn(&Bytes, Duration, &Span);
type NewOnEos = fn(Option<&HeaderMap>, Duration, &Span);
type NewOnFailure = fn(StatusInRangeFailureClass, Duration, &Span);
type NewTraceLayer = TraceLayer<
    SharedClassifier<StatusInRangeAsFailures>,
    NewMakeSpan,
    NewOnRequest,
    NewOnResponse,
    NewOnBodyChunk,
    NewOnEos,
    NewOnFailure,
>;

// Tracing middleware.
pub(crate) static TRACING_MIDDLEWARE: LazyLock<NewTraceLayer> = LazyLock::new(|| {
    let classifier = StatusInRangeAsFailures::new_for_client_and_server_errors();
    TraceLayer::new(classifier.into_make_classifier())
        .make_span_with(new_make_span as NewMakeSpan)
        .on_request(new_on_request as NewOnRequest)
        .on_response(new_on_response as NewOnResponse)
        .on_body_chunk(new_on_body_chunk as NewOnBodyChunk)
        .on_eos(new_on_eos as NewOnEos)
        .on_failure(new_on_failure as NewOnFailure)
});

fn new_make_span(request: &Request<Body>) -> Span {
    let uri = request.uri();
    let headers = request.headers();
    tracing::info_span!(
        "HTTP request",
        "otel.kind" = "server",
        "otel.name" = crate::AxumCluster::name(),
        "http.method" = request.method().as_str(),
        "http.scheme" = uri.scheme_str(),
        "http.target" = uri.path_and_query().map(|p| p.as_str()),
        "http.client_ip" = headers.get_client_ip().map(|ip| ip.to_string()),
        "http.user_agent" = headers.get_str("user-agent"),
        "http.request.header.traceparent" = Empty,
        "http.request.header.tracestate" = Empty,
        "http.response.header.traceparent" = Empty,
        "http.response.header.tracestate" = Empty,
        "http.status_code" = Empty,
        "http.server.duration" = Empty,
        "net.host.name" = uri.host(),
        "net.host.port" = uri.port_u16(),
        "context.request_id" = Empty,
        "context.session_id" = Empty,
        "context.span_id" = Empty,
        "context.trace_id" = Empty,
        "context.parent_id" = Empty,
    )
}

fn new_on_request(request: &Request<Body>, span: &Span) {
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

fn new_on_response(response: &Response<BoxBody>, latency: Duration, span: &Span) {
    let headers = response.headers();
    let traceparent = headers.get_str("traceparent");
    span.record("http.response.header.traceparent", traceparent);
    span.record(
        "http.response.header.tracestate",
        headers.get_str("tracestate"),
    );
    span.record(
        "context.trace_id",
        traceparent
            .and_then(TraceContext::from_traceparent)
            .map(|ctx| Uuid::from_u128(ctx.trace_id()).to_string()),
    );
    span.record("context.request_id", headers.get_str("x-request-id"));
    span.record("http.status_code", response.status().as_u16());
    span.record(
        "http.server.duration",
        u64::try_from(latency.as_millis()).ok(),
    );
    tracing::info!("finished processing request");
}

fn new_on_body_chunk(chunk: &Bytes, _latency: Duration, _span: &Span) {
    tracing::debug!("flushed {} bytes", chunk.len());
}

fn new_on_eos(_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span) {
    tracing::debug!(
        stream_duration = u64::try_from(stream_duration.as_millis()).ok(),
        "end of stream",
    );
}

fn new_on_failure(error: StatusInRangeFailureClass, latency: Duration, span: &Span) {
    span.record(
        "http.server.duration",
        u64::try_from(latency.as_millis()).ok(),
    );
    match error {
        StatusInRangeFailureClass::StatusCode(status_code) => {
            span.record("http.status_code", status_code.as_u16());
            tracing::error!("response failed");
        }
        StatusInRangeFailureClass::Error(err) => {
            tracing::error!(err);
        }
    }
}
