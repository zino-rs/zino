use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use tracing::{field::Empty, Span};
use tracing_actix_web::{RootSpanBuilder, TracingLogger};
use zino_core::{application::Application, trace::TraceContext, Uuid};

/// Tracing middleware.
#[inline]
pub(crate) fn tracing_middleware() -> TracingLogger<NewRootSpanBuilder> {
    TracingLogger::new()
}

/// Root span builder.
pub(crate) struct NewRootSpanBuilder;

impl RootSpanBuilder for NewRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let uri = request.uri();
        let headers = request.headers();
        let traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
        let tracestate = headers.get("tracestate").and_then(|v| v.to_str().ok());
        let trace_context = traceparent.and_then(TraceContext::from_traceparent);
        let parent_id = trace_context
            .and_then(|ctx| ctx.parent_id())
            .map(|parent_id| format!("{parent_id:x}"));
        tracing::info_span!(
            "HTTP request",
            "otel.kind" = "server",
            "otel.name" = crate::Cluster::name(),
            "otel.status_code" = Empty,
            "http.scheme" = uri.scheme_str(),
            "http.method" = request.method().as_str(),
            "http.route" = request.match_pattern(),
            "http.target" = uri.path_and_query().map(|p| p.as_str()),
            "http.client_ip" = request.connection_info().realip_remote_addr(),
            "http.user_agent" = headers.get("user-agent").and_then(|v| v.to_str().ok()),
            "http.request.header.traceparent" = traceparent,
            "http.request.header.tracestate" = tracestate,
            "http.response.header.traceparent" = Empty,
            "http.response.header.tracestate" = Empty,
            "http.status_code" = Empty,
            "http.server.duration" = Empty,
            "net.host.name" = uri.host(),
            "net.host.port" = uri.port_u16(),
            "context.session_id" = headers.get("session-id").and_then(|v| v.to_str().ok()),
            "context.trace_id" = Empty,
            "context.request_id" = Empty,
            "context.span_id" = Empty,
            "context.parent_id" = parent_id,
        )
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        match &outcome {
            Ok(response) => {
                let res = response.response();
                let headers = res.headers();
                let traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
                span.record("http.response.header.traceparent", traceparent);
                span.record(
                    "http.response.header.tracestate",
                    headers.get("tracestate").and_then(|v| v.to_str().ok()),
                );
                span.record(
                    "context.trace_id",
                    traceparent
                        .and_then(TraceContext::from_traceparent)
                        .map(|ctx| Uuid::from_u128(ctx.trace_id()).to_string()),
                );
                span.record(
                    "context.request_id",
                    headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                );
                span.record(
                    "context.span_id",
                    span.id().map(|id| format!("{:x}", id.into_u64())),
                );
                if res.error().is_none() {
                    span.record("http.status_code", res.status().as_u16());
                    span.record("otel.status_code", "OK");
                    tracing::info!("finished processing request");
                } else {
                    let status_code = response.status();
                    span.record("http.status_code", status_code.as_u16());
                    if status_code.is_client_error() {
                        span.record("otel.status_code", "OK");
                    } else {
                        span.record("otel.status_code", "ERROR");
                    }
                }
            }
            Err(error) => {
                let status_code = error.as_response_error().status_code();
                if status_code.is_client_error() {
                    span.record("otel.status_code", "OK");
                } else {
                    span.record("otel.status_code", "ERROR");
                }
            }
        };
    }
}
