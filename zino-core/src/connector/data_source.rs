use self::DataSourceConnector::*;
use super::Connector;
use crate::{extend::TomlTableExt, BoxError, Map, Record};
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
    /// Protocol.
    protocol: &'static str,
    /// Data souce type
    data_source_type: String,
    /// Name
    name: String,
    /// Catalog
    catalog: String,
    /// Connector
    connector: DataSourceConnector,
}

impl DataSource {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(
        protocol: &'static str,
        data_source_type: Option<String>,
        name: impl Into<String>,
        catalog: impl Into<String>,
        connector: DataSourceConnector,
    ) -> Self {
        Self {
            protocol,
            data_source_type: data_source_type.unwrap_or_else(|| protocol.to_owned()),
            name: name.into(),
            catalog: catalog.into(),
            connector,
        }
    }

    /// Constructs a new instance with the protocol and configuration,
    /// returning an error if it fails.
    ///
    /// Currently, we have built-in support for the following protocols:
    ///
    /// - `arrow`
    /// - `http`
    /// - `mssql`
    /// - `mysql`
    /// - `postgres`
    /// - `sqlite`
    /// - `taos`
    pub fn try_new(protocol: &'static str, config: &Table) -> Result<DataSource, BoxError> {
        let mut data_source = match protocol {
            #[cfg(feature = "connector-arrow")]
            "arrow" => ArrowConnector::try_new_data_source(config)?,
            #[cfg(feature = "connector-http")]
            "http" => HttpConnector::try_new_data_source(config)?,
            #[cfg(feature = "connector-mssql")]
            "mssql" => MssqlPool::try_new_data_source(config)?,
            #[cfg(feature = "connector-mysql")]
            "mysql" => MySqlPool::try_new_data_source(config)?,
            #[cfg(feature = "connector-postgres")]
            "postgres" => PgPool::try_new_data_source(config)?,
            #[cfg(feature = "connector-sqlite")]
            "sqlite" => SqlitePool::try_new_data_source(config)?,
            #[cfg(feature = "connector-taos")]
            "taos" => TaosPool::try_new_data_source(config)?,
            _ => return Err(format!("data source protocol `{protocol}` is unsupported").into()),
        };
        let data_source_type = config.get_str("type").unwrap_or(protocol);
        data_source.data_source_type = data_source_type.to_owned();
        Ok(data_source)
    }

    /// Returns the protocol.
    pub fn protocol(&self) -> &'static str {
        &self.protocol
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the catalog.
    #[inline]
    pub fn catalog(&self) -> &str {
        self.catalog.as_str()
    }
}

impl Connector for DataSource {
    fn try_new_data_source(config: &Table) -> Result<DataSource, BoxError> {
        let data_source_type = config.get_str("type").unwrap_or("unkown");
        let protocol = match data_source_type {
            "arrow" => "arrow",
            "http" | "rest" | "graphql" => "http",
            "mssql" => "mssql",
            "mysql" | "ceresdb" | "databend" | "mariadb" | "tidb" => "mysql",
            "postgres" | "citus" | "greptimedb" | "hologres" | "opengauss" | "postgis"
            | "timescaledb" => "postgres",
            "sqlite" => "sqlite",
            "taos" => "taos",
            _ => {
                if let Some(protocol) = config.get_str("protocol") {
                    protocol.to_owned().leak()
                } else {
                    return Err(
                        format!("data source type `{data_source_type}` is unsupported").into(),
                    );
                }
            }
        };
        Self::try_new(protocol, config)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, BoxError> {
        match &self.connector {
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

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, BoxError> {
        match &self.connector {
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

    async fn query_one(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<Record>, BoxError> {
        match &self.connector {
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
}
