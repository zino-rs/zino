use crate::AxumCluster;
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
use zino_core::{application::Application, trace::TraceContext, Uuid};

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
        "otel.name" = AxumCluster::name(),
        "http.method" = request.method().as_str(),
        "http.scheme" = uri.scheme_str(),
        "http.target" = uri.path_and_query().map(|t| t.as_str()),
        "http.user_agent" = headers.get("user-agent").and_then(|v| v.to_str().ok()),
        "http.request.header.traceparent" = Empty,
        "http.response.header.traceparent" = Empty,
        "http.status_code" = Empty,
        "http.server.duration" = Empty,
        "net.host.name" = uri.host(),
        "net.host.port" = uri.port_u16(),
        "zino.request_id" = Empty,
        "zino.trace_id" = Empty,
        "zino.session_id" = Empty,
        id = Empty,
    )
}

fn new_on_request(request: &Request<Body>, span: &Span) {
    let headers = request.headers();
    span.record(
        "http.request.header.traceparent",
        headers.get("traceparent").and_then(|v| v.to_str().ok()),
    );
    span.record(
        "zino.session_id",
        headers.get("session-id").and_then(|v| v.to_str().ok()),
    );
    span.record("id", span.id().map(|t| t.into_u64()));
    tracing::debug!("started processing request");
}

fn new_on_response(response: &Response<BoxBody>, latency: Duration, span: &Span) {
    let headers = response.headers();
    let traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
    span.record("http.response.header.traceparent", traceparent);
    span.record(
        "zino.trace_id",
        traceparent
            .and_then(TraceContext::from_traceparent)
            .map(|ctx| Uuid::from_u128(ctx.trace_id()).to_string()),
    );
    span.record(
        "zino.request_id",
        headers.get("x-request-id").and_then(|v| v.to_str().ok()),
    );
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
