use axum::{
    body::Body,
    http::{
        header::{ETAG, IF_NONE_MATCH},
        Request, StatusCode,
    },
    middleware::Next,
    response::Response,
};

pub(crate) async fn extract_etag(req: Request<Body>, next: Next<Body>) -> Response {
    if req.method().is_idempotent() {
        let req_etag = req.headers().get(IF_NONE_MATCH).cloned();
        let mut res = next.run(req).await;
        if let Some(etag) = res.headers_mut().remove("x-etag") {
            if req_etag.as_ref() == Some(&etag) && res.status().is_success() {
                *res.status_mut() = StatusCode::NOT_MODIFIED;
            }
            res.headers_mut().insert(ETAG, etag);
        }
        res
    } else {
        next.run(req).await
    }
}
