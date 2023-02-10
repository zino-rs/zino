//! Database connectors.
//!
//! Supported data sources:
//! - `ceresdb`: CeresDB
//! - `citus`: Citus
//! - `databend`: Databend
//! - `hologres`: Aliyun Hologres
//! - `mariadb`: MariaDB
//! - `mssql`: MSSQL
//! - `mysql`: MySQL
//! - `opengauss`: openGauss
//! - `postgis`: PostGIS
//! - `postgres`: PostgreSQL
//! - `sqlite`: SQLite
//! - `taos`: TDengine
//! - `tidb`: TiDB
//! - `timescaledb`: TimescaleDB
//!

use crate::{extend::TomlTableExt, state::State, BoxError, Map};
use std::{collections::HashMap, sync::LazyLock};
use toml::Table;

mod data_source;
mod sqlx_common;

/// Supported connectors.
mod mssql_connector;
mod mysql_connector;
mod postgres_connector;
mod sqlite_connector;
mod taos_connector;

pub use data_source::DataSource;

use data_source::DataSourcePool;
use sqlx_common::impl_sqlx_connector;

/// Underlying trait of all data sources for implementors.
trait Connector {
    /// Creates a new data source with the configuration.
    fn new_data_source(config: &'static Table) -> Result<DataSource, BoxError>;

    /// Executes the query and returns the total number of rows affected.
    async fn execute(&self, sql: &str, params: Option<Map>) -> Result<Option<u64>, BoxError>;

    /// Executes the query in the table, and parses it as `Vec<Map>`.
    async fn query(&self, sql: &str, params: Option<Map>) -> Result<Vec<Map>, BoxError>;

    /// Executes the query in the table, and parses it as a `Map`.
    async fn query_one(&self, sql: &str, params: Option<Map>) -> Result<Option<Map>, BoxError>;
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
