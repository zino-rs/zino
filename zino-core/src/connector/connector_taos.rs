use super::{Connector, DataSource, DataSourcePool::Taos};
use crate::{database::Query, extend::TomlTableExt, state::State, BoxError, Map, Record};
use futures::TryStreamExt;
use taos::{AsyncFetchable, AsyncQueryable, PoolBuilder, TBuilder, TaosBuilder, TaosPool};
use toml::Table;

impl Connector for TaosPool {
    fn new_data_source(config: &'static Table) -> Result<DataSource, BoxError> {
        let name = config.get_str("name").unwrap_or("taos");
        let database = config.get_str("database").unwrap_or_default();
        let authority = State::format_authority(config, Some(6041));
        let dsn = format!("taos+ws://{authority}/{database}");

        let max_size = config.get_u32("max-size").unwrap_or(5000);
        let min_idle = config.get_u32("min-idle").unwrap_or(2);
        let pool_options = PoolBuilder::new()
            .max_size(max_size)
            .min_idle(Some(min_idle))
            .max_lifetime(None);
        let pool = TaosBuilder::from_dsn(dsn)?.with_pool_builder(pool_options)?;
        let data_source = DataSource::new("taos", name, database, Taos(pool));
        Ok(data_source)
    }

    async fn execute(&self, sql: &str, params: Option<Map>) -> Result<Option<u64>, BoxError> {
        let taos = self.get()?;
        let sql = Query::format_sql(sql, params);
        let affected_rows = taos.exec(sql).await?;
        Ok(affected_rows.try_into().ok())
    }

    async fn query(&self, sql: &str, params: Option<Map>) -> Result<Vec<Record>, BoxError> {
        let taos = self.get()?;
        let sql = Query::format_sql(sql, params);
        let mut result = taos.query(sql).await?;
        let mut data = Vec::new();
        let mut rows = result.rows();
        while let Some(row) = rows.try_next().await? {
            let mut record = Record::new();
            for (name, value) in row {
                record.push((name.to_owned(), value.to_json_value().into()));
            }
            data.push(record);
        }
        Ok(data)
    }

    async fn query_one(&self, sql: &str, params: Option<Map>) -> Result<Option<Record>, BoxError> {
        let taos = self.get()?;
        let sql = Query::format_sql(sql, params);
        let mut result = taos.query(sql).await?;
        let data = result.rows().try_next().await?.map(|row| {
            let mut record = Record::new();
            for (name, value) in row {
                record.push((name.to_owned(), value.to_json_value().into()));
            }
            record
        });
        Ok(data)
    }
}
