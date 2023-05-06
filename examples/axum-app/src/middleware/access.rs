use axum::{body::Body, middleware::Next, response::Response};
use zino::{Request, Result};

pub(crate) async fn check_client_ip(req: Request, next: Next<Body>) -> Result<Response> {
    Ok(next.run(req.into()).await)
}
