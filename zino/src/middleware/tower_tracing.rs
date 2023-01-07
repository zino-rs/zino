use std::sync::LazyLock;
use tower_http::{
    classify::{SharedClassifier, StatusInRangeAsFailures},
    trace::{
        DefaultMakeSpan, DefaultOnBodyChunk, DefaultOnEos, DefaultOnFailure, DefaultOnRequest,
        DefaultOnResponse, TraceLayer,
    },
    LatencyUnit,
};
use tracing::Level;

// Tracing middleware.
pub(crate) static TRACING_MIDDLEWARE: LazyLock<
    TraceLayer<SharedClassifier<StatusInRangeAsFailures>>,
> = LazyLock::new(|| {
    let classifier = StatusInRangeAsFailures::new_for_client_and_server_errors();
    TraceLayer::new(classifier.into_make_classifier())
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(true),
        )
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        )
        .on_body_chunk(DefaultOnBodyChunk::new())
        .on_eos(
            DefaultOnEos::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        )
        .on_failure(
            DefaultOnFailure::new()
                .level(Level::ERROR)
                .latency_unit(LatencyUnit::Micros),
        )
});
