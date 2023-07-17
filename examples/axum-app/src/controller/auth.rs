use crate::service;
use zino::{prelude::*, Request, Response, Result};
use zino_model::User;

pub async fn login(mut req: Request) -> Result {
    let body: Map = req.parse_body().await?;
    let (user_id, token) = service::auth::generate_token(body).await.extract(&req)?;

    let mut mutations = Map::from_entry("status", "Active");
    let (validation, user) = User::update_by_id(&user_id, &mut mutations, None)
        .await
        .extract(&req)?;
    if !validation.is_success() {
        return Err(Rejection::bad_request(validation).context(&req).into());
    }

    let mut data = Map::data_entry(user.snapshot());
    data.upsert("access_token", token);

    let mut res = Response::default().context(&req);
    res.set_data(&data);
    Ok(res.into())
}
