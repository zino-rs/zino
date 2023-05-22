use crate::service::task;
use zino::{prelude::*, Request, Response, Result};

pub async fn execute(mut req: Request) -> Result {
    let mut query = Query::default();
    let mut res: Response = req.query_validation(&mut query)?;
    let body: Map = req.parse_body().await?;
    let data = task::execute_query(&query, body).await.extract(&req)?;
    res.set_data(&data);
    Ok(res.into())
}
