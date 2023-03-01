use zino::{
    ExtractRejection, JsonObjectExt, Map, Query, Request, RequestContext, Response, Schema,
};
use zino_core::connector::{DataFrameExecutor, GlobalConnector};
use zino_model::User;

pub(crate) async fn execute(mut req: Request) -> zino::Result {
    let mut query = Query::new();
    let mut res: Response = req.query_validation(&mut query)?;
    let records = User::find(&query).await.extract_with_context(&req)?;
    let body: Map = req.parse_body().await?;
    let connector = GlobalConnector::get("mock")
        .and_then(|data_source| data_source.get_arrow_connector())
        .ok_or("fail to get an Arrow connector for the `mock` data souce")
        .extract_with_context(&req)?;
    let df = connector
        .read_avro_records(records.as_slice())
        .await
        .extract_with_context(&req)?
        .select_columns(&["name"])
        .extract_with_context(&req)?;

    let sql = body.get_str("sql").unwrap_or("SELECT 'ok' AS status;");
    let data: Vec<Map> = connector
        .try_get_session_context()
        .await
        .extract_with_context(&req)?
        .sql(sql)
        .await
        .extract_with_context(&req)?
        .union(df)
        .extract_with_context(&req)?
        .query_as()
        .await
        .extract_with_context(&req)?;
    res.set_data(&data);
    Ok(res.into())
}
