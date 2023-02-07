//! Database connectors.

use crate::{extend::TomlTableExt, state::State};
use sqlx::{
    mssql::{MssqlConnectOptions, MssqlPool, MssqlPoolOptions},
    mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions},
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
    Error,
};
use std::{collections::HashMap, sync::LazyLock};
use toml::Table;

mod data_source;

pub use data_source::DataSource;

use data_source::DataSourcePool;

/// Underlying trait of all data sources for implementors.
trait Connector {
    /// Creates a new data source with the configuration.
    fn new_data_source(config: &'static Table) -> DataSource;

    /// Executes the query and returns the total number of rows affected.
    async fn execute<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<u64, Error>;
}

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
}

impl Connector for MySqlPool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("mysql");
        let database = config.get_str("database").unwrap_or_default();
        let connect_options = MySqlConnectOptions::new();
        let pool = MySqlPoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::MySql(pool))
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
}

impl Connector for PgPool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("postgres");
        let database = config.get_str("database").unwrap_or("postgres");
        let connect_options = PgConnectOptions::new();
        let pool = PgPoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::Postgres(pool))
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
}

impl Connector for SqlitePool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("sqlite");
        let database = config.get_str("database").unwrap_or_default();
        let connect_options = SqliteConnectOptions::new();
        let pool = SqlitePoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::Sqlite(pool))
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
