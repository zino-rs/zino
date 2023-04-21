use axum::{body::Body, middleware::Next, response::Response};
use zino::{Error, Rejection, Request, RequestContext, Result};

pub(crate) async fn check_client_ip(req: Request, next: Next<Body>) -> Result<Response> {
    if let Some(client_ip) = req.client_ip() {
        let message = format!("client ip `{client_ip} is blocked`");
        return Err(Rejection::forbidden(Error::new(message)).into());
    }
    Ok(next.run(req.into()).await)
}
