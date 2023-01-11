use axum::{
    body::{Body, BoxBody},
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use zino_core::RequestContext;

pub(crate) async fn request_context(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<BoxBody>, StatusCode> {
    let mut request = crate::AxumExtractor(req);
    let ext = match request.get_context() {
        Some(_) => None,
        None => {
            let mut ctx = request.new_context();
            let original_uri = request.original_uri().await;
            ctx.set_request_path(original_uri.path());
            Some(ctx)
        }
    };

    let mut req = request.0;
    if let Some(ctx) = ext {
        req.extensions_mut().insert(ctx);
    }
    Ok(next.run(req).await)
}
