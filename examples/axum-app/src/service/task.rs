use zino::{Error, JsonObjectExt, Map, Query, Schema};
use zino_core::connector::{DataFrameExecutor, GlobalConnector};
use zino_model::User;

pub(crate) async fn execute_union_query(query: &Query, body: Map) -> Result<Vec<Map>, Error> {
    let records = User::find(&query).await?;
    let connector = GlobalConnector::get("mock")
        .and_then(|data_source| data_source.get_arrow_connector())
        .ok_or_else(|| Error::new("fail to get an Arrow connector for the `mock` data souce"))?;
    let df = connector
        .read_avro_records(records.as_slice())
        .await?
        .select_columns(&["name"])?;

    let sql = body.get_str("sql").unwrap_or("SELECT 'ok' AS status;");
    let data: Vec<Map> = connector
        .try_get_session_context()
        .await?
        .sql(sql)
        .await?
        .union(df)?
        .query_as()
        .await?;
    Ok(data)
}
