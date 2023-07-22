use crate::service::auth;
use zino::{prelude::*, Request, Response, Result};
use zino_model::User;

pub async fn login(mut req: Request) -> Result {
    let body: Map = req.parse_body().await?;
    let (user_id, mut data) = auth::generate_token(body).await.extract(&req)?;

    let mut mutations = Map::from_entry("status", "Active");
    let (validation, user) = User::update_by_id(&user_id, &mut mutations, None)
        .await
        .extract(&req)?;
    if !validation.is_success() {
        return Err(Rejection::bad_request(validation).context(&req).into());
    }
    data.upsert("entry", user.snapshot());

    let mut res = Response::default().context(&req);
    res.set_data(&data);
    Ok(res.into())
}

pub async fn refresh(req: Request) -> Result {
    let claims = req.parse_jwt_claims(JwtClaims::shared_key())?;
    let data = auth::refresh_token(&claims).await.extract(&req)?;
    let mut res = Response::default().context(&req);
    res.set_data(&data);
    Ok(res.into())
}

pub async fn logout(req: Request) -> Result {
    let user_session = req
        .get_data::<UserSession<Uuid>>()
        .ok_or_else(|| Error::new("401 Unauthorized: the user session is invalid"))
        .extract(&req)?;

    let mut mutations = Map::from_entry("status", "Inactive");
    let user_id = user_session.user_id();
    let (validation, user) = User::update_by_id(user_id, &mut mutations, None)
        .await
        .extract(&req)?;
    if !validation.is_success() {
        return Err(Rejection::bad_request(validation).context(&req).into());
    }

    let data = Map::data_entry(user.snapshot());
    let mut res = Response::default().context(&req);
    res.set_data(&data);
    Ok(res.into())
}
