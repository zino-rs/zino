use axum::{body::Body, middleware::Next, response::Response};
use zino::{prelude::*, Request, Result};

pub async fn init_user_session(mut req: Request, next: Next<Body>) -> Result<Response> {
    if let Ok(claims) = req.parse_jwt_claims(JwtClaims::shared_key()) &&
        let Ok(mut user_session) = UserSession::<Uuid>::try_from_jwt_claims(claims)
    {
        if let Ok(session_id) = req.parse_session_id() {
            user_session.set_session_id(session_id);
        }
        req.set_data(user_session);
    }
    Ok(next.run(req.into()).await)
}
