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
pub(crate) fn tracing_middleware() -> TracingLogger<CustomRootSpanBuilder> {
    TracingLogger::new()
}

/// Root span builder.
pub(crate) struct CustomRootSpanBuilder;

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let name = crate::Cluster::name();
        let method = request.method();
        let route = request.match_pattern();

        // Client IP
        let connection_info = request.connection_info();
        let client_ip = connection_info.realip_remote_addr();

        // URI
        let uri = request.uri();
        let scheme = uri.scheme_str();
        let host = uri.host();
        let port = uri.port_u16();
        let path = uri.path();
        let query = uri.query();

        // Headers
        let headers = request.headers();
        let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());
        let traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
        let tracestate = headers.get("tracestate").and_then(|v| v.to_str().ok());
        let trace_context = traceparent.and_then(TraceContext::from_traceparent);
        let parent_id = trace_context
            .and_then(|ctx| ctx.parent_id())
            .map(|parent_id| format!("{parent_id:x}"));
        let session_id = headers.get("session-id").and_then(|v| v.to_str().ok());

        if method.is_safe() {
            tracing::info_span!(
                "HTTP request",
                "otel.kind" = "server",
                "otel.name" = name,
                "otel.status_code" = Empty,
                "url.scheme" = scheme,
                "url.path" = path,
                "url.query" = query,
                "http.route" = route,
                "http.request.method" = method.as_str(),
                "http.request.header.traceparent" = traceparent,
                "http.request.header.tracestate" = tracestate,
                "http.response.header.traceparent" = Empty,
                "http.response.header.tracestate" = Empty,
                "http.response.header.server_timing" = Empty,
                "http.response.status_code" = Empty,
                "client.address" = client_ip,
                "server.address" = host,
                "server.port" = port,
                "user_agent.original" = user_agent,
                "context.session_id" = session_id,
                "context.trace_id" = Empty,
                "context.request_id" = Empty,
                "context.span_id" = Empty,
                "context.parent_id" = parent_id,
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
                "http.route" = route,
                "http.request.method" = method.as_str(),
                "http.request.header.traceparent" = traceparent,
                "http.request.header.tracestate" = tracestate,
                "http.response.header.traceparent" = Empty,
                "http.response.header.tracestate" = Empty,
                "http.response.header.server_timing" = Empty,
                "http.response.status_code" = Empty,
                "client.address" = client_ip,
                "server.address" = host,
                "server.port" = port,
                "user_agent.original" = user_agent,
                "context.session_id" = session_id,
                "context.trace_id" = Empty,
                "context.request_id" = Empty,
                "context.span_id" = Empty,
                "context.parent_id" = parent_id,
            )
        }
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
                    "http.response.header.server_timing",
                    headers.get("server-timing").and_then(|v| v.to_str().ok()),
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
                    span.record("http.response.status_code", res.status().as_u16());
                    span.record("otel.status_code", "OK");
                    tracing::info!("finished processing request");
                } else {
                    let status_code = response.status();
                    span.record("http.response.status_code", status_code.as_u16());
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
