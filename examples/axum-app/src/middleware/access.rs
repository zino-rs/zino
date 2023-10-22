use crate::model::User;
use axum::{body::Body, middleware::Next, response::Response};
use zino::{prelude::*, Request, Result};
use zino_model::user::JwtAuthService;

pub async fn init_user_session(mut req: Request, next: Next<Body>) -> Result<Response> {
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
