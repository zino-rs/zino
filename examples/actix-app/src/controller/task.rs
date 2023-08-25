use crate::service;
use zino::{prelude::*, Request, Result};

pub async fn execute(mut req: Request) -> Result {
    let mut query = Query::default();
    let mut res = req.query_validation(&mut query)?;
    let body: Map = req.parse_body().await?;
    let data = service::task::execute_query(&query, body)
        .await
        .extract(&req)?;
    res.set_json_data(data);
    Ok(res.into())
}
