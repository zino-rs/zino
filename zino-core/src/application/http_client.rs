use crate::application::Application;
use reqwest::{Client, Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension, Result};
use reqwest_tracing::{
    default_on_request_end, reqwest_otel_span, OtelName, ReqwestOtelSpanBackend, TracingMiddleware,
};
use std::{sync::OnceLock, time::Instant};
use task_local_extensions::Extensions;
use tracing::Span;

pub(super) fn init<APP: Application + ?Sized>() {
    let name = APP::name();
    let version = APP::version();
    let reqwest_client = Client::builder()
        .user_agent(format!("ZinoBot/1.0 {name}/{version}"))
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .unwrap_or_else(|err| panic!("failed to create an HTTP client: {err}"));
    let client = ClientBuilder::new(reqwest_client)
        .with_init(Extension(OtelName("zino-bot".into())))
        .with(TracingMiddleware::default())
        .with(TracingMiddleware::<RequestTiming>::new())
        .build();
    SHARED_HTTP_CLIENT
        .set(client)
        .expect("failed to set an HTTP client for the application");
}

/// Request timing.
struct RequestTiming;

impl ReqwestOtelSpanBackend for RequestTiming {
    fn on_request_start(req: &Request, extension: &mut Extensions) -> Span {
        extension.insert(Instant::now());
        reqwest_otel_span!(
            name = "example-request",
            req,
            time_elapsed = tracing::field::Empty
        )
    }

    fn on_request_end(span: &Span, outcome: &Result<Response>, extensions: &mut Extensions) {
        let latency_micros = extensions
            .get::<Instant>()
            .and_then(|t| u64::try_from(t.elapsed().as_micros()).ok());
        default_on_request_end(span, outcome);
        span.record("latency_micros", latency_micros);
    }
}

/// Shared HTTP client.
pub(crate) static SHARED_HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();
