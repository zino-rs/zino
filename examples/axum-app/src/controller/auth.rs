use crate::model::{User, UserColumn::*};
use zino::{Request, Response, Result, prelude::*};

pub async fn login(mut req: Request) -> Result {
    let credentials = req.parse_body::<BasicCredentials>().await?;
    let query = QueryBuilder::new()
        .and_eq(Account, credentials.username())
        .and_not_in(Status, ["Locked", "Deleted"])
        .build();
    let user: User = User::find_one_as(&query).await.extract(&req)?;
    if !user.verify(credentials.password()) {
        reject!(req, unauthorized, "invalid user account or password");
    }

    let user_id = user.id();
    let user_info = Map::from_entry("roles", user.roles());
    let claims = JwtClaims::with_data(user_id, user_info);
    let mut data = claims.refreshable_bearer_auth().extract(&req)?;

    let mut mutation = MutationBuilder::<User>::new()
        .set(Status, "Active")
        .set(LastLoginAt, user.current_login_at())
        .set_if_nonempty(LastLoginIp, user.current_login_ip())
        .set_if_some(CurrentLoginIp, req.client_ip())
        .set_now(CurrentLoginAt)
        .inc_one(LoginCount)
        .set_now(UpdatedAt)
        .inc_one(Version)
        .build();
    let user: User = User::update_by_id(user_id, &mut mutation)
        .await
        .extract(&req)?;
    data.upsert("entry", user.snapshot());

    let mut res = Response::default().context(&req);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn refresh(req: Request) -> Result {
    let user_id = req
        .parse_jwt_claims(JwtClaims::shared_key())?
        .parse_refresh_token::<i64>()
        .extract(&req)?;
    let query = QueryBuilder::new()
        .field(Roles)
        .primary_key(user_id)
        .and_not_in(Status, ["SignedOut", "Locked", "Deleted"])
        .build();
    let user_info: Map = User::find_one(&query).await.extract(&req)?;
    let claims = JwtClaims::with_data(user_id, user_info);
    let data = claims.bearer_auth().extract(&req)?;
    let mut res = Response::default().context(&req);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn logout(req: Request) -> Result {
    let user_session = req.get_data::<UserSession<_>>().extract(&req)?;
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
