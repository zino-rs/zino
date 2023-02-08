use super::{Connector, DataSource, DataSourcePool, SerializeRow};
use crate::{extend::TomlTableExt, Map};
use futures::TryStreamExt;
use serde_json::Value;
use sqlx::{
    mssql::{MssqlConnectOptions, MssqlPool, MssqlPoolOptions},
    Error,
};
use toml::Table;

impl Connector for MssqlPool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("mssql");
        let database = config.get_str("database").unwrap_or("master");
        let connect_options = MssqlConnectOptions::new();
        let pool = MssqlPoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::Mssql(pool))
    }

    async fn execute<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<u64, Error> {
        let mut query = sqlx::query(sql);
        if let Some(params) = params {
            for param in params {
                query = query.bind(param);
            }
        }
        let query_result = query.execute(self).await?;
        Ok(query_result.rows_affected())
    }

    async fn query<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Vec<Map>, Error> {
        let mut query = sqlx::query(sql);
        if let Some(params) = params {
            for param in params {
                query = query.bind(param);
            }
        }
        let mut rows = query.fetch(self);
        let mut data = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let value = serde_json::to_value(&SerializeRow(row))
                .map_err(|err| Error::Decode(err.into()))?;
            if let Value::Object(map) = value {
                data.push(map);
            }
        }
        Ok(data)
    }

    async fn query_one<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Option<Map>, Error> {
        let mut query = sqlx::query(sql);
        if let Some(params) = params {
            for param in params {
                query = query.bind(param);
            }
        }
        let data = match query.fetch_optional(self).await? {
            Some(row) => {
                let value = serde_json::to_value(&SerializeRow(row))
                    .map_err(|err| Error::Decode(err.into()))?;
                if let Value::Object(map) = value {
                    Some(map)
                } else {
                    None
                }
            }
            None => None,
        };
        Ok(data)
    }
}
