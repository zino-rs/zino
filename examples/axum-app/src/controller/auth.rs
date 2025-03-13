use crate::model::{User, UserColumn::*};
use zino::{Request, Response, Result, prelude::*};
use zino_model::user::JwtAuthService;

pub async fn login(mut req: Request) -> Result {
    let body: Map = req.parse_body().await?;
    let (user_id, mut data) = User::generate_token(body).await.extract(&req)?;

    let last_login_ip = data.remove("current_login_ip");
    let last_login_at = data
        .remove("current_login_at")
        .and_then(|v| v.as_date_time());
    let mut mutation = MutationBuilder::<User>::new()
        .set(Status, "Active")
        .set_if_not_null(LastLoginIp, last_login_ip)
        .set_if_some(LastLoginAt, last_login_at)
        .set_if_some(CurrentLoginIp, req.client_ip())
        .set_now(CurrentLoginAt)
        .inc_one(LoginCount)
        .set_now(UpdatedAt)
        .inc_one(Version)
        .build();
    let user: User = User::update_by_id(&user_id, &mut mutation)
        .await
        .extract(&req)?;
    data.upsert("entry", user.snapshot());

    let mut res = Response::default().context(&req);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn refresh(req: Request) -> Result {
    let claims = req.parse_jwt_claims(JwtClaims::shared_key())?;
    let data = User::refresh_token(&claims).await.extract(&req)?;
    let mut res = Response::default().context(&req);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn logout(req: Request) -> Result {
    let user_session = req
        .get_data::<UserSession<_>>()
        .ok_or_else(|| warn!("401 Unauthorized: user session is invalid"))
        .extract(&req)?;
    let user_id = user_session.user_id();

    let mut mutation = MutationBuilder::<User>::new()
        .set(Status, "SignedOut")
        .set_now(UpdatedAt)
        .inc_one(Version)
        .build();
    let user: User = User::update_by_id(user_id, &mut mutation)
        .await
        .extract(&req)?;

    let mut res = Response::default().context(&req);
    res.set_json_data(Map::data_entry(user.snapshot()));
    Ok(res.into())
}
