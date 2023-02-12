//! Unified database connector to different data sources.
//!
//! ## Supported data sources
//!
//! | Data source type | Database               | Feature flag           |
//! |------------------|------------------------|------------------------|
//! | `hologres`       | Aliyun Hologres        | `connector-postgres`   |
//! | `ceresdb`        | CeresDB                | `connector-postgres`   |
//! | `citus`          | Citus                  | `connector-postgres`   |
//! | `databend`       | Databend               | `connector-mysql`      |
//! | `hologres`       | Aliyun Hologres        | `connector-postgres`   |
//! | `maridb`         | MariaDB                | `connector-mysql`      |
//! | `mssql`          | MSSQL (SQL Server)     | `connector-mssql`      |
//! | `mysql`          | MySQL                  | `connector-mysql`      |
//! | `opengauss`      | openGauss              | `connector-postgres`   |
//! | `postgis`        | PostGIS                | `connector-postgres`   |
//! | `postgres`       | PostgreSQL             | `connector-postgres`   |
//! | `sqlite`         | SQLite                 | `connector-sqlite`     |
//! | `taos`           | TDengine               | `connector-taos`       |
//! | `tidb`           | TiDB                   | `connector-mysql`      |
//! | `timescaledb`    | TimescaleDB            | `connector-postgres`   |
//!

use crate::{extend::TomlTableExt, state::State, BoxError, Map, Record};
use std::{collections::HashMap, sync::LazyLock};
use toml::Table;

mod data_source;
mod sqlx_common;

/// Supported connectors.
#[cfg(feature = "connector-mssql")]
mod connector_mssql;
#[cfg(feature = "connector-mysql")]
mod connector_mysql;
#[cfg(feature = "connector-postgres")]
mod connector_postgres;
#[cfg(feature = "connector-sqlite")]
mod connector_sqlite;
#[cfg(feature = "connector-taos")]
mod connector_taos;

pub use data_source::DataSource;

use data_source::DataSourcePool;
use sqlx_common::impl_sqlx_connector;

/// Underlying trait of all data sources for implementors.
pub trait Connector {
    /// Creates a new data source with the configuration.
    fn new_data_source(config: &'static Table) -> Result<DataSource, BoxError>;

    /// Executes the query and returns the total number of rows affected.
    async fn execute(&self, sql: &str, params: Option<Map>) -> Result<Option<u64>, BoxError>;

    /// Executes the query in the table, and parses it as `Vec<Map>`.
    async fn query(&self, sql: &str, params: Option<Map>) -> Result<Vec<Record>, BoxError>;

    /// Executes the query in the table, and parses it as a `Map`.
    async fn query_one(&self, sql: &str, params: Option<Map>) -> Result<Option<Record>, BoxError>;
}

/// Global database connector.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalConnector;

impl GlobalConnector {
    /// Gets the data source for the specific database service.
    #[inline]
    pub fn get(name: &'static str) -> Option<&'static DataSource> {
        GLOBAL_CONNECTOR.get(name)
    }
}

/// Global database connector.
static GLOBAL_CONNECTOR: LazyLock<HashMap<&'static str, DataSource>> = LazyLock::new(|| {
    let mut data_sources = HashMap::new();
    if let Some(connectors) = State::shared().config().get_array("connector") {
        for connector in connectors.iter().filter_map(|v| v.as_table()) {
            let database_type = connector.get_str("type").unwrap_or("unkown");
            let name = connector.get_str("name").unwrap_or(database_type);
            let data_source = DataSource::new_connector(database_type, connector)
                .unwrap_or_else(|err| panic!("failed to connect data source `{name}`: {err}"));
            data_sources.insert(name, data_source);
        }
    }
    data_sources
});
