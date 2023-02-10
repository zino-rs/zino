use self::DataSourcePool::*;
use super::Connector;
use crate::{BoxError, Map};
use serde::de::DeserializeOwned;
use sqlx::{mssql::MssqlPool, mysql::MySqlPool, postgres::PgPool, sqlite::SqlitePool};
use taos::TaosPool;
use toml::Table;

/// Supported data source pool.
#[non_exhaustive]
pub(super) enum DataSourcePool {
    /// MSSQL
    Mssql(MssqlPool),
    /// MySQL
    MySql(MySqlPool),
    /// Postgres
    Postgres(PgPool),
    /// SQLite
    Sqlite(SqlitePool),
    /// TDengine
    Taos(TaosPool),
}

/// Data sources.
pub struct DataSource {
    /// Database type
    database_type: &'static str,
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
        database_type: &'static str,
        name: &'static str,
        database: &'static str,
        pool: DataSourcePool,
    ) -> Self {
        Self {
            database_type,
            name,
            database,
            pool,
        }
    }

    /// Creates a new connector with the configuration for the specific database service.
    pub fn new_connector(
        database_type: &'static str,
        config: &'static Table,
    ) -> Result<Self, BoxError> {
        match database_type {
            "mssql" => {
                let mut data_source = MssqlPool::new_data_source(config)?;
                data_source.database_type = database_type;
                Ok(data_source)
            }
            "mysql" | "ceresdb" | "databend" | "mariadb" | "tidb" => {
                let mut data_source = MySqlPool::new_data_source(config)?;
                data_source.database_type = database_type;
                Ok(data_source)
            }
            "postgres" | "citus" | "hologres" | "opengauss" | "postgis" | "timescaledb" => {
                let mut data_source = PgPool::new_data_source(config)?;
                data_source.database_type = database_type;
                Ok(data_source)
            }
            "sqlite" => SqlitePool::new_data_source(config),
            "taos" => TaosPool::new_data_source(config),
            _ => Err(format!("database type `{database_type}` is unsupported").into()),
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
            Mssql(pool) => pool.execute(sql, params).await,
            MySql(pool) => pool.execute(sql, params).await,
            Postgres(pool) => pool.execute(sql, params).await,
            Sqlite(pool) => pool.execute(sql, params).await,
            Taos(pool) => pool.execute(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as `Vec<Map>`.
    pub async fn query(&self, sql: &str, params: Option<Map>) -> Result<Vec<Map>, BoxError> {
        match &self.pool {
            Mssql(pool) => pool.query(sql, params).await,
            MySql(pool) => pool.query(sql, params).await,
            Postgres(pool) => pool.query(sql, params).await,
            Sqlite(pool) => pool.query(sql, params).await,
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
        serde_json::from_value(data.into()).map_err(|err| err.into())
    }

    /// Executes the query in the table, and parses it as a `Map`.
    pub async fn query_one(&self, sql: &str, params: Option<Map>) -> Result<Option<Map>, BoxError> {
        match &self.pool {
            Mssql(pool) => pool.query_one(sql, params).await,
            MySql(pool) => pool.query_one(sql, params).await,
            Postgres(pool) => pool.query_one(sql, params).await,
            Sqlite(pool) => pool.query_one(sql, params).await,
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
            Some(data) => serde_json::from_value(data.into()).map_err(|err| err.into()),
            None => Ok(None),
        }
    }
}
