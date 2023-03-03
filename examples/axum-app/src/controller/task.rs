use crate::service::task;
use zino::{ExtractRejection, Map, Query, Request, RequestContext, Response};

pub(crate) async fn execute(mut req: Request) -> zino::Result {
    let mut query = Query::new();
    let mut res: Response = req.query_validation(&mut query)?;
    let body: Map = req.parse_body().await?;
    let data = task::execute_union_query(&query, body)
        .await
        .extract_with_context(&req)?;
    res.set_data(&data);
    Ok(res.into())
}
