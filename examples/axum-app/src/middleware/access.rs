use axum::{body::Body, middleware::Next, response::Response};
use zino::{prelude::*, Request, Result};

pub async fn init_user_session(mut req: Request, next: Next<Body>) -> Result<Response> {
    if let Ok(claims) = req.parse_jwt_claims(JwtClaims::shared_key()) &&
        let Some(user_id) = claims.subject().and_then(|s| s.parse().ok())
    {
        let roles = claims.data().parse_array("roles").unwrap_or_default();
        let session_id = req.parse_session_id().ok();
        let mut user_session = UserSession::new(user_id, session_id);
        user_session.set_roles(roles);
        req.set_data::<UserSession<Uuid, String>>(user_session);
    }
    Ok(next.run(req.into()).await)
}
