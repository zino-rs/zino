use self::DataSourcePool::*;
use super::Connector;
use crate::{extend::AvroRecordExt, BoxError, Map, Record};
use apache_avro::types::Value;
use serde::de::DeserializeOwned;
use toml::Table;

#[cfg(feature = "connector-mssql")]
use sqlx::mssql::MssqlPool;
#[cfg(feature = "connector-mysql")]
use sqlx::mysql::MySqlPool;
#[cfg(feature = "connector-postgres")]
use sqlx::postgres::PgPool;
#[cfg(feature = "connector-sqlite")]
use sqlx::sqlite::SqlitePool;
#[cfg(feature = "connector-taos")]
use taos::TaosPool;

/// Supported data source pool.
#[non_exhaustive]
pub(super) enum DataSourcePool {
    #[cfg(feature = "connector-mssql")]
    /// MSSQL
    Mssql(MssqlPool),
    #[cfg(feature = "connector-mysql")]
    /// MySQL
    MySql(MySqlPool),
    #[cfg(feature = "connector-postgres")]
    /// Postgres
    Postgres(PgPool),
    #[cfg(feature = "connector-sqlite")]
    /// SQLite
    Sqlite(SqlitePool),
    #[cfg(feature = "connector-taos")]
    /// TDengine
    Taos(TaosPool),
}

/// Data sources.
pub struct DataSource {
    /// Data souce type
    data_source_type: &'static str,
    /// Name
    name: &'static str,
    /// Database
    database: &'static str,
    /// Pool
    pool: DataSourcePool,
}

impl DataSource {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(
        data_source_type: &'static str,
        name: &'static str,
        database: &'static str,
        pool: DataSourcePool,
    ) -> Self {
        Self {
            data_source_type,
            name,
            database,
            pool,
        }
    }

    /// Creates a new connector with the configuration for the specific data source.
    pub fn new_connector(
        data_source_type: &'static str,
        config: &'static Table,
    ) -> Result<Self, BoxError> {
        match data_source_type {
            #[cfg(feature = "connector-mssql")]
            "mssql" => {
                let mut data_source = MssqlPool::new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-mysql")]
            "mysql" | "ceresdb" | "databend" | "mariadb" | "tidb" => {
                let mut data_source = MySqlPool::new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-postgres")]
            "postgres" | "citus" | "hologres" | "opengauss" | "postgis" | "timescaledb" => {
                let mut data_source = PgPool::new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-sqlite")]
            "sqlite" => SqlitePool::new_data_source(config),
            #[cfg(feature = "connector-taos")]
            "taos" => TaosPool::new_data_source(config),
            _ => Err(format!("data source type `{data_source_type}` is unsupported").into()),
        }
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the database.
    #[inline]
    pub fn database(&self) -> &'static str {
        self.database
    }

    /// Executes the query and returns the total number of rows affected.
    pub async fn execute(&self, sql: &str, params: Option<Map>) -> Result<Option<u64>, BoxError> {
        match &self.pool {
            #[cfg(feature = "connector-mssql")]
            Mssql(pool) => pool.execute(sql, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.execute(sql, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.execute(sql, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.execute(sql, params).await,
            #[cfg(feature = "connector-taos")]
            Taos(pool) => pool.execute(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as `Vec<Map>`.
    pub async fn query(&self, sql: &str, params: Option<Map>) -> Result<Vec<Record>, BoxError> {
        match &self.pool {
            #[cfg(feature = "connector-mssql")]
            Mssql(pool) => pool.query(sql, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.query(sql, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.query(sql, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.query(sql, params).await,
            #[cfg(feature = "connector-taos")]
            Taos(pool) => pool.query(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as `Vec<T>`.
    pub async fn query_as<T: DeserializeOwned>(
        &self,
        sql: &str,
        params: Option<Map>,
    ) -> Result<Vec<T>, BoxError> {
        let data = self.query(sql, params).await?;
        let value = data
            .into_iter()
            .map(|record| record.into_avro_map())
            .collect::<Vec<_>>();
        apache_avro::from_value(&Value::Array(value)).map_err(|err| err.into())
    }

    /// Executes the query in the table, and parses it as a `Map`.
    pub async fn query_one(
        &self,
        sql: &str,
        params: Option<Map>,
    ) -> Result<Option<Record>, BoxError> {
        match &self.pool {
            #[cfg(feature = "connector-mssql")]
            Mssql(pool) => pool.query_one(sql, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.query_one(sql, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.query_one(sql, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.query_one(sql, params).await,
            #[cfg(feature = "connector-taos")]
            Taos(pool) => pool.query_one(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as an instance of type `T`.
    pub async fn query_one_as<T: DeserializeOwned>(
        &self,
        sql: &str,
        params: Option<Map>,
    ) -> Result<Option<T>, BoxError> {
        match self.query_one(sql, params).await? {
            Some(data) => {
                let value = Value::Union(1, Box::new(data.into_avro_map()));
                apache_avro::from_value(&value).map_err(|err| err.into())
            }
            None => Ok(None),
        }
    }
}
