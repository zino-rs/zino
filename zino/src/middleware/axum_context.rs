use axum::{
    body::{Body, BoxBody},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use zino_core::request::RequestContext;

pub(crate) async fn request_context(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<BoxBody>, StatusCode> {
    let request = crate::AxumExtractor(req);
    let new_context = request
        .get_context()
        .is_none()
        .then(|| request.new_context());

    let mut req = request.0;
    if let Some(ctx) = new_context {
        req.extensions_mut().insert(ctx);
    }
    Ok(next.run(req).await)
}
