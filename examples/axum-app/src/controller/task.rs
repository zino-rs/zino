use zino::{ExtractRejection, JsonObjectExt, Map, Request, RequestContext, Response};
use zino_core::connector::{Connector, GlobalConnector};

pub(crate) async fn execute(mut req: Request) -> zino::Result {
    let mut res = Response::default().provide_context(&req);
    let data_source = GlobalConnector::get("mock")
        .ok_or("fail to get the `mock` data souce")
        .extract_with_context(&req)?;
    let body: Map = req.parse_body().await.extract_with_context(&req)?;
    let sql = body.get_str("sql").unwrap_or("SELECT 'ok' AS status;");
    let data: Vec<Map> = data_source
        .query_as(sql, None)
        .await
        .extract_with_context(&req)?;
    res.set_data(&data);
    Ok(res.into())
}
