use axum::{body::Body, middleware::Next, response::Response};
use zino::{prelude::*, Request, Result};
use zino_model::user::{JwtAuthService, User};

pub async fn init_user_session(mut req: Request, next: Next<Body>) -> Result<Response> {
    if let Ok(claims) = req.parse_jwt_claims(JwtClaims::shared_key()) {
        match User::verify_jwt_claims(&claims).await {
            Ok(verified) => {
                if verified &&
                    let Ok(mut user_session) = UserSession::<Uuid>::try_from_jwt_claims(claims)
                {
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
    } else if req.request_method() == "POST" {
        reject!(req, unauthorized, "login is required");
    }
    Ok(next.run(req.into()).await)
}
