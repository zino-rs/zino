use axum::{http, middleware::Next, response::Response};
use tracing::Span;
use zino_core::request::RequestContext;

pub(crate) async fn request_context(req: crate::Request, next: Next) -> Response {
    let new_context = req.get_context().is_none().then(|| req.new_context());

    let mut req = http::Request::from(req);
    if let Some(ctx) = new_context {
        Span::current().record("context.request_id", ctx.request_id().to_string());
        req.extensions_mut().insert(ctx);
    }
    next.run(req).await
}
