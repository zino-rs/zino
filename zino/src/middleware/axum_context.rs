use axum::{body::Body, http, middleware::Next, response::Response};
use zino_core::request::RequestContext;

pub(crate) async fn request_context(req: crate::Request, next: Next<Body>) -> Response {
    let new_context = req.get_context().is_none().then(|| req.new_context());

    let mut req = http::Request::from(req);
    if let Some(ctx) = new_context {
        req.extensions_mut().insert(ctx);
    }
    next.run(req).await
}
