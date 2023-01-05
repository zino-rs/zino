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
    let mut req_extractor = crate::AxumExtractor(req);
    let ext = match req_extractor.get_context() {
        Some(_) => None,
        None => {
            let mut ctx = req_extractor.new_context();
            let original_uri = req_extractor.original_uri().await;
            ctx.set_request_path(original_uri.path());
            Some(ctx)
        }
    };

    let mut req = req_extractor.0;
    if let Some(ctx) = ext {
        req.extensions_mut().insert(ctx);
    }
    Ok(next.run(req).await)
}
