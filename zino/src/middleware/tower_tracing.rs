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
use tracing_subscriber::fmt::{time, writer::MakeWriterExt};
use zino_core::State;

// Tracing middleware.
pub(crate) static TRACING_MIDDLEWARE: LazyLock<
    TraceLayer<SharedClassifier<StatusInRangeAsFailures>>,
> = LazyLock::new(|| {
    let shared_state = State::shared();
    let app_env = shared_state.env();
    let is_dev = app_env == "dev";

    let mut env_filter = if is_dev {
        "sqlx=trace,tower_http=trace,zino=trace,zino_core=trace"
    } else {
        "sqlx=warn,tower_http=info,zino=info,zino_core=info"
    };
    let mut display_target = is_dev;
    let mut display_filename = false;
    let mut display_line_number = false;
    let mut display_thread_names = false;
    let mut display_span_list = false;
    let display_current_span = true;
    let include_headers = true;

    let config = shared_state.config();
    if let Some(tracing) = config.get("tracing").and_then(|t| t.as_table()) {
        if let Some(filter) = tracing.get("filter").and_then(|t| t.as_str()) {
            env_filter = filter;
        }
        display_target = tracing
            .get("display-target")
            .and_then(|t| t.as_bool())
            .unwrap_or(is_dev);
        display_filename = tracing
            .get("display-filename")
            .and_then(|t| t.as_bool())
            .unwrap_or(false);
        display_line_number = tracing
            .get("display-line-number")
            .and_then(|t| t.as_bool())
            .unwrap_or(false);
        display_thread_names = tracing
            .get("display-thread-names")
            .and_then(|t| t.as_bool())
            .unwrap_or(false);
        display_span_list = tracing
            .get("display-span-list")
            .and_then(|t| t.as_bool())
            .unwrap_or(false);
    }

    let stderr = std::io::stderr.with_max_level(Level::WARN);
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(display_target)
        .with_file(display_filename)
        .with_line_number(display_line_number)
        .with_thread_names(display_thread_names)
        .with_span_list(display_span_list)
        .with_current_span(display_current_span)
        .with_timer(time::LocalTime::rfc_3339())
        .map_writer(move |w| stderr.or_else(w))
        .init();

    let classifier = StatusInRangeAsFailures::new_for_client_and_server_errors();
    TraceLayer::new(classifier.into_make_classifier())
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(include_headers),
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
