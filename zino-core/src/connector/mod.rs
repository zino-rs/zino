//! Unified connector to different data sources.
//!
//! ## Supported data sources
//!
//! | Data source type | Description            | Feature flag           |
//! |------------------|------------------------|------------------------|
//! | `arrow`          | Apache Arrow           | `connector-arrow`      |
//! | `ceresdb`        | CeresDB                | `connector-mysql`      |
//! | `citus`          | Citus                  | `connector-postgres`   |
//! | `databend`       | Databend               | `connector-mysql`      |
//! | `graphql`        | GraphQL API            | `connector-http`       |
//! | `greptimedb`     | GreptimeDB             | `connector-postgres`   |
//! | `highgo`         | HighGo Database        | `connector-postgres`   |
//! | `hologres`       | Aliyun Hologres        | `connector-postgres`   |
//! | `http`           | HTTP services          | `connector-http`       |
//! | `mariadb`        | MariaDB                | `connector-mysql`      |
//! | `mysql`          | MySQL                  | `connector-mysql`      |
//! | `opengauss`      | openGauss              | `connector-postgres`   |
//! | `postgis`        | PostGIS                | `connector-postgres`   |
//! | `postgres`       | PostgreSQL             | `connector-postgres`   |
//! | `rest`           | RESTful API            | `connector-http`       |
//! | `sqlite`         | SQLite                 | `connector-sqlite`     |
//! | `tidb`           | TiDB                   | `connector-mysql`      |
//! | `timescaledb`    | TimescaleDB            | `connector-postgres`   |
//!

use crate::{
    application::StaticRecord, error::Error, extension::TomlTableExt, state::State, AvroValue,
    LazyLock, Map, Record,
};
use serde::de::DeserializeOwned;
use toml::Table;

mod data_source;

pub use data_source::DataSource;
use data_source::DataSourceConnector;

/// Supported connectors.
#[cfg(feature = "connector-arrow")]
mod connector_arrow;
#[cfg(feature = "connector-http")]
mod connector_http;
#[cfg(feature = "connector-mysql")]
mod connector_mysql;
#[cfg(feature = "connector-postgres")]
mod connector_postgres;
#[cfg(feature = "connector-sqlite")]
mod connector_sqlite;
#[cfg(any(
    feature = "connector-mysql",
    feature = "connector-postgres",
    feature = "connector-sqlite"
))]
mod sqlx_common;

#[cfg(feature = "connector-arrow")]
pub use connector_arrow::{ArrowConnector, DataFrameExecutor};
#[cfg(feature = "connector-http")]
pub use connector_http::HttpConnector;

/// Underlying trait of all data sources for implementors.
pub trait Connector {
    /// Constructs a new data source with the configuration,
    /// returning an error if it fails.
    fn try_new_data_source(config: &Table) -> Result<DataSource, Error>;

    /// Executes the query and returns the total number of rows affected.
    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error>;

    /// Executes the query and parses it as `Vec<Record>`.
    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, Error>;

    /// Executes the query and parses it as `Vec<T>`.
    async fn query_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let data = self.query(query, params).await?;
        let value = data.into_iter().map(AvroValue::Record).collect();
        apache_avro::from_value(&AvroValue::Array(value)).map_err(|err| err.into())
    }

    /// Executes the query and parses it as a `Record`.
    async fn query_one(&self, query: &str, params: Option<&Map>) -> Result<Option<Record>, Error>;

    /// Executes the query and parses it as an instance of type `T`.
    async fn query_one_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        if let Some(record) = self.query_one(query, params).await? {
            let value = AvroValue::Union(1, Box::new(AvroValue::Record(record)));
            apache_avro::from_value(&value).map_err(|err| err.into())
        } else {
            Ok(None)
        }
    }
}

/// Global access to the shared data source connectors.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalConnector;

impl GlobalConnector {
    /// Gets the data source for the specific service.
    #[inline]
    pub fn get(name: &str) -> Option<&'static DataSource> {
        SHARED_DATA_SOURCE_CONNECTORS.find(name)
    }
}

/// Shared connectors.
static SHARED_DATA_SOURCE_CONNECTORS: LazyLock<StaticRecord<DataSource>> = LazyLock::new(|| {
    let mut data_sources = StaticRecord::new();
    if let Some(connectors) = State::shared().config().get_array("connector") {
        for connector in connectors.iter().filter_map(|v| v.as_table()) {
            let data_source_type = connector.get_str("type").unwrap_or("unkown");
            let name = connector.get_str("name").unwrap_or(data_source_type);
            let data_source = DataSource::try_new_data_source(connector)
                .unwrap_or_else(|err| panic!("fail to connect data source `{name}`: {err}"));
            data_sources.add(name, data_source);
        }
    }
    data_sources
});
