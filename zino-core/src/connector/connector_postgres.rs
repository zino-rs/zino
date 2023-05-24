use super::{sqlx_common::SerializeRow, Connector, DataSource, DataSourceConnector::Postgres};
use crate::{error::Error, extension::TomlTableExt, format, state::State, Map, Record};
use apache_avro::types::Value;
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use toml::Table;

impl Connector for PgPool {
    fn try_new_data_source(config: &Table) -> Result<DataSource, Error> {
        let name = config.get_str("name").unwrap_or("postgres");
        let database = config.get_str("database").unwrap_or("postgres");
        let authority = State::format_authority(config, Some(5432));
        let dsn = format!("postgres://{authority}/{database}");

        let max_connections = config.get_u32("max-connections").unwrap_or(16);
        let min_connections = config.get_u32("min-connections").unwrap_or(2);
        let max_lifetime = config
            .get_duration("max-lifetime")
            .unwrap_or_else(|| Duration::from_secs(60 * 60));
        let idle_timeout = config
            .get_duration("idle-timeout")
            .unwrap_or_else(|| Duration::from_secs(10 * 60));
        let acquire_timeout = config
            .get_duration("acquire-timeout")
            .unwrap_or_else(|| Duration::from_secs(30));
        let pool_options = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(max_lifetime)
            .idle_timeout(idle_timeout)
            .acquire_timeout(acquire_timeout);
        let pool = pool_options.connect_lazy(&dsn)?;
        let data_source = DataSource::new("postgres", None, name, database, Postgres(pool));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error> {
        let (sql, values) = format::query::prepare_sql_query(query, params, '$');
        let mut query = sqlx::query(&sql);
        for value in values {
            query = query.bind(value.to_string());
        }

        let query_result = query.execute(self).await?;
        Ok(Some(query_result.rows_affected()))
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, Error> {
        let (sql, values) = format::query::prepare_sql_query(query, params, '$');
        let mut query = sqlx::query(&sql);
        for value in values {
            query = query.bind(value.to_string());
        }

        let mut rows = query.fetch(self);
        let mut records = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let value = apache_avro::to_value(&SerializeRow(row))?;
            if let Value::Record(record) = value {
                records.push(record);
            }
        }
        Ok(records)
    }

    async fn query_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let (sql, values) = format::query::prepare_sql_query(query, params, '$');
        let mut query = sqlx::query(&sql);
        for value in values {
            query = query.bind(value.to_string());
        }

        let mut rows = query.fetch(self);
        let mut data = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let json_value = serde_json::to_value(&SerializeRow(row))?;
            let value = serde_json::from_value(json_value)?;
            data.push(value);
        }
        Ok(data)
    }

    async fn query_one(&self, query: &str, params: Option<&Map>) -> Result<Option<Record>, Error> {
        let (sql, values) = format::query::prepare_sql_query(query, params, '$');
        let mut query = sqlx::query(&sql);
        for value in values {
            query = query.bind(value.to_string());
        }

        let data = if let Some(row) = query.fetch_optional(self).await? {
            let value = apache_avro::to_value(&SerializeRow(row))?;
            if let Value::Record(record) = value {
                Some(record)
            } else {
                None
            }
        } else {
            None
        };
        Ok(data)
    }

    async fn query_one_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        let (sql, values) = format::query::prepare_sql_query(query, params, '$');
        let mut query = sqlx::query(&sql);
        for value in values {
            query = query.bind(value.to_string());
        }

        if let Some(row) = query.fetch_optional(self).await? {
            let json_value = serde_json::to_value(&SerializeRow(row))?;
            serde_json::from_value(json_value).map_err(Error::from)
        } else {
            Ok(None)
        }
    }
}
