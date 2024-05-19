use crate::model::User;
use axum::{middleware::Next, response::Response};
use zino::{prelude::*, Request, Result};
use zino_model::user::JwtAuthService;

pub async fn init_user_session(mut req: Request, next: Next) -> Result<Response> {
    let claims = req
        .parse_jwt_claims(JwtClaims::shared_key())
        .map_err(|rejection| rejection.context(&req))?;
    match User::verify_jwt_claims(&claims).await {
        Ok(verified) => {
            if verified {
                let mut user_session =
                    UserSession::<i64>::try_from_jwt_claims(claims).extract(&req)?;
                if let Ok(session_id) = req.parse_session_id() {
                    user_session.set_session_id(session_id);
                }
                req.set_data(user_session);
            } else {
                reject!(req, unauthorized, "invalid JWT claims");
            }
        }
        Err(err) => reject!(req, unauthorized, err),
    }
    Ok(next.run(req.into()).await)
}

pub async fn check_admin_role(req: Request, next: Next) -> Result<Response> {
    if req.request_method() == "POST" {
        if let Some(user_session) = req.get_data::<UserSession<i64>>() {
            if !user_session.has_role("admin") {
                reject!(req, unauthorized, "a role of `admin` is required");
            }
        }
    }
    Ok(next.run(req.into()).await)
}
