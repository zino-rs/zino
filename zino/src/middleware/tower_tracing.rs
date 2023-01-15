use axum::{
    body::{Body, BoxBody, Bytes},
    http::{HeaderMap, Request, Response},
};
use http_types::{trace::TraceContext, Trailers};
use std::{sync::LazyLock, time::Duration};
use tower_http::{
    classify::{SharedClassifier, StatusInRangeAsFailures, StatusInRangeFailureClass},
    trace::TraceLayer,
};
use tracing::{field::Empty, Span};
use zino_core::Uuid;

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
    tracing::info_span!(
        "http-request",
        method = request.method().as_str(),
        path = uri.path(),
        query = uri.query(),
        span_id = Empty,
        request_id = Empty,
        trace_id = Empty,
        session_id = Empty,
    )
}

fn new_on_request(request: &Request<Body>, span: &Span) {
    let headers = request.headers();
    let session_id = headers.get("session-id").and_then(|v| v.to_str().ok());
    span.record("session_id", session_id);
    span.record("span_id", span.id().map(|t| t.into_u64()));

    tracing::debug!("started processing request");
}

fn new_on_response(response: &Response<BoxBody>, latency: Duration, span: &Span) {
    let headers = response.headers();
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    span.record("request_id", request_id);

    let trace_id = headers
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(|traceparent| {
            let mut trailers = Trailers::new();
            trailers.insert("traceparent", traceparent);

            TraceContext::from_headers(&*trailers).ok().flatten()
        })
        .map(|trace_context| Uuid::from_u128(trace_context.trace_id()).to_string());
    span.record("trace_id", trace_id);
    tracing::info!(
        status = response.status().as_u16(),
        latency_micros = u64::try_from(latency.as_micros()).ok(),
        "finished processing request",
    );
}

fn new_on_body_chunk(chunk: &Bytes, _latency: Duration, _span: &Span) {
    tracing::debug!(chunk_size = chunk.len(), "sending body chunk");
}

fn new_on_eos(_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span) {
    tracing::debug!(
        stream_duration_micros = u64::try_from(stream_duration.as_micros()).ok(),
        "end of stream",
    );
}

fn new_on_failure(error: StatusInRangeFailureClass, latency: Duration, _span: &Span) {
    let latency = u64::try_from(latency.as_micros()).ok();
    match error {
        StatusInRangeFailureClass::StatusCode(status_code) => {
            tracing::error!(
                status = status_code.as_u16(),
                latency_micros = latency,
                "response failed",
            );
        }
        StatusInRangeFailureClass::Error(err) => {
            tracing::error!(latency_micros = latency, err);
        }
    }
}
