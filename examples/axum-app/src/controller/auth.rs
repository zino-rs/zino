use crate::service;
use zino::{prelude::*, Request, Response, Result};

pub async fn login(mut req: Request) -> Result {
    let body: Map = req.parse_body().await?;
    let (user, token) = service::auth::generate_token(body).await.extract(&req)?;
    let mut data = Map::data_entry(user);
    data.upsert("access_token", token);

    let mut res = Response::default().context(&req);
    res.set_data(&data);
    Ok(res.into())
}
