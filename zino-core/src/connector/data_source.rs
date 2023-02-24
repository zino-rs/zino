use self::DataSourceConnector::*;
use super::Connector;
use crate::{extend::AvroRecordExt, BoxError, Map, Record};
use apache_avro::types::Value;
use serde::de::DeserializeOwned;
use toml::Table;

#[cfg(feature = "connector-arrow")]
use super::ArrowConnector;
#[cfg(feature = "connector-http")]
use super::HttpConnector;
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

/// Supported data source connectors.
#[non_exhaustive]
pub(super) enum DataSourceConnector {
    #[cfg(feature = "connector-arrow")]
    /// Apache Arrow
    Arrow(ArrowConnector),
    #[cfg(feature = "connector-http")]
    /// HTTP
    Http(HttpConnector),
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
    /// Name
    name: &'static str,
    /// Data souce type
    data_source_type: &'static str,
    /// Catalog
    catalog: &'static str,
    /// Pool
    pool: DataSourceConnector,
}

impl DataSource {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(
        name: &'static str,
        data_source_type: &'static str,
        catalog: &'static str,
        pool: DataSourceConnector,
    ) -> Self {
        Self {
            name,
            data_source_type,
            catalog,
            pool,
        }
    }

    /// Constructs a new instance with the configuration for the specific data source,
    /// returning an error if it fails.
    pub fn try_new(
        data_source_type: &'static str,
        config: &'static Table,
    ) -> Result<Self, BoxError> {
        match data_source_type {
            #[cfg(feature = "connector-arrow")]
            "arrow" => ArrowConnector::try_new_data_source(config),
            #[cfg(feature = "connector-http")]
            "http" | "rest" | "graphql" => {
                let mut data_source = HttpConnector::try_new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-mssql")]
            "mssql" => {
                let mut data_source = MssqlPool::try_new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-mysql")]
            "mysql" | "ceresdb" | "databend" | "mariadb" | "tidb" => {
                let mut data_source = MySqlPool::try_new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-postgres")]
            "postgres" | "citus" | "hologres" | "opengauss" | "postgis" | "timescaledb" => {
                let mut data_source = PgPool::try_new_data_source(config)?;
                data_source.data_source_type = data_source_type;
                Ok(data_source)
            }
            #[cfg(feature = "connector-sqlite")]
            "sqlite" => SqlitePool::try_new_data_source(config),
            #[cfg(feature = "connector-taos")]
            "taos" => TaosPool::try_new_data_source(config),
            _ => Err(format!("data source type `{data_source_type}` is unsupported").into()),
        }
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the catalog.
    #[inline]
    pub fn catalog(&self) -> &'static str {
        self.catalog
    }

    /// Executes the query and returns the total number of rows affected.
    pub async fn execute(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<u64>, BoxError> {
        match &self.pool {
            #[cfg(feature = "connector-arrow")]
            Arrow(connector) => connector.execute(query, params).await,
            #[cfg(feature = "connector-http")]
            Http(connector) => connector.execute(query, params).await,
            #[cfg(feature = "connector-mssql")]
            Mssql(pool) => pool.execute(query, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.execute(query, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.execute(query, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.execute(query, params).await,
            #[cfg(feature = "connector-taos")]
            Taos(pool) => pool.execute(query, params).await,
        }
    }

    /// Executes the query and parses it as `Vec<Map>`.
    pub async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, BoxError> {
        match &self.pool {
            #[cfg(feature = "connector-arrow")]
            Arrow(connector) => connector.query(query, params).await,
            #[cfg(feature = "connector-http")]
            Http(connector) => connector.query(query, params).await,
            #[cfg(feature = "connector-mssql")]
            Mssql(pool) => pool.query(query, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.query(query, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.query(query, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.query(query, params).await,
            #[cfg(feature = "connector-taos")]
            Taos(pool) => pool.query(query, params).await,
        }
    }

    /// Executes the query and parses it as `Vec<T>`.
    pub async fn query_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, BoxError> {
        let data = self.query(query, params).await?;
        let value = data
            .into_iter()
            .map(|record| Value::Map(record.into_avro_map()))
            .collect::<Vec<_>>();
        apache_avro::from_value(&Value::Array(value)).map_err(|err| err.into())
    }

    /// Executes the query and parses it as a `Map`.
    pub async fn query_one(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<Record>, BoxError> {
        match &self.pool {
            #[cfg(feature = "connector-arrow")]
            Arrow(connector) => connector.query_one(query, params).await,
            #[cfg(feature = "connector-http")]
            Http(connector) => connector.query_one(query, params).await,
            #[cfg(feature = "connector-mssql")]
            Mssql(pool) => pool.query_one(query, params).await,
            #[cfg(feature = "connector-mysql")]
            MySql(pool) => pool.query_one(query, params).await,
            #[cfg(feature = "connector-postgres")]
            Postgres(pool) => pool.query_one(query, params).await,
            #[cfg(feature = "connector-sqlite")]
            Sqlite(pool) => pool.query_one(query, params).await,
            #[cfg(feature = "connector-taos")]
            Taos(pool) => pool.query_one(query, params).await,
        }
    }

    /// Executes the query and parses it as an instance of type `T`.
    pub async fn query_one_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, BoxError> {
        if let Some(data) = self.query_one(query, params).await? {
            let value = Value::Union(1, Box::new(Value::Map(data.into_avro_map())));
            apache_avro::from_value(&value).map_err(|err| err.into())
        } else {
            Ok(None)
        }
    }
}
