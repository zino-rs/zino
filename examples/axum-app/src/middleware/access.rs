use axum::{middleware::Next, response::Response};
use zino::{Request, Result, prelude::*};

pub async fn init_user_session(mut req: Request, next: Next) -> Result<Response> {
    let claims = req
        .parse_jwt_claims(JwtClaims::shared_key())
        .map_err(|rejection| rejection.context(&req))?;
    let session = UserSession::<i64>::try_from_jwt_claims(claims).extract(&req)?;
    req.set_data(session);
    Ok(next.run(req.into()).await)
}

pub async fn check_admin_role(req: Request, next: Next) -> Result<Response> {
    if req.request_method() == "POST"
        && let Some(session) = req.get_data::<UserSession<i64>>()
        && !session.has_role("admin")
    {
        reject!(req, unauthorized, "a role of `admin` is required");
    }
    Ok(next.run(req.into()).await)
}
