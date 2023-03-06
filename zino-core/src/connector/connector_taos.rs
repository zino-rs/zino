use super::{Connector, DataSource, DataSourceConnector::Taos};
use crate::{error::Error, extend::TomlTableExt, format, state::State, Map, Record};
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use taos::{AsyncFetchable, AsyncQueryable, PoolBuilder, TBuilder, TaosBuilder, TaosPool};
use toml::Table;

impl Connector for TaosPool {
    fn try_new_data_source(config: &Table) -> Result<DataSource, Error> {
        let name = config.get_str("name").unwrap_or("taos");
        let database = config.get_str("database").unwrap_or(name);
        let authority = State::format_authority(config, Some(6041));
        let dsn = format!("taos+ws://{authority}/{database}");

        let max_size = config.get_u32("max-size").unwrap_or(5000);
        let min_idle = config.get_u32("min-idle").unwrap_or(2);
        let pool_options = PoolBuilder::new()
            .max_size(max_size)
            .min_idle(Some(min_idle))
            .max_lifetime(None);
        let pool = TaosBuilder::from_dsn(dsn)?.with_pool_builder(pool_options)?;
        let data_source = DataSource::new("taos", None, name, database, Taos(pool));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error> {
        let taos = self.get()?;
        let sql = format::format_query(query, params);
        let affected_rows = taos.exec(sql).await?;
        Ok(affected_rows.try_into().ok())
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, Error> {
        let taos = self.get()?;
        let sql = format::format_query(query, params);
        let mut result = taos.query(sql).await?;
        let mut rows = result.rows();
        let mut records = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let mut record = Record::new();
            for (name, value) in row {
                record.push((name.to_owned(), value.to_json_value().into()));
            }
            records.push(record);
        }
        Ok(records)
    }

    async fn query_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let taos = self.get()?;
        let sql = format::format_query(query, params);
        let mut result = taos.query(sql).await?;
        let mut rows = result.rows();
        let mut data = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let mut map = Map::new();
            for (name, value) in row {
                map.insert(name.to_owned(), value.to_json_value());
            }
            data.push(map);
        }
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    async fn query_one(&self, query: &str, params: Option<&Map>) -> Result<Option<Record>, Error> {
        let taos = self.get()?;
        let sql = format::format_query(query, params);
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

    async fn query_one_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        let taos = self.get()?;
        let sql = format::format_query(query, params);
        let mut result = taos.query(sql).await?;
        if let Some(row) = result.rows().try_next().await? {
            let mut map = Map::new();
            for (name, value) in row {
                map.insert(name.to_owned(), value.to_json_value());
            }
            serde_json::from_value(map.into()).map_err(Error::from)
        } else {
            Ok(None)
        }
    }
}
