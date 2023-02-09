use self::DataSourcePool::*;
use super::Connector;
use crate::{BoxError, Map};
use serde::de::DeserializeOwned;
use sqlx::{mssql::MssqlPool, mysql::MySqlPool, postgres::PgPool, sqlite::SqlitePool};
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
}

/// Data sources.
pub struct DataSource {
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
    pub(super) fn new(name: &'static str, database: &'static str, pool: DataSourcePool) -> Self {
        Self {
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
            "mssql" => Ok(MssqlPool::new_data_source(config)),
            "mysql" => Ok(MySqlPool::new_data_source(config)),
            "postgres" => Ok(PgPool::new_data_source(config)),
            "sqlite" => Ok(SqlitePool::new_data_source(config)),
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
    pub async fn execute<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Option<u64>, BoxError> {
        match &self.pool {
            Mssql(pool) => pool.execute(sql, params).await,
            MySql(pool) => pool.execute(sql, params).await,
            Postgres(pool) => pool.execute(sql, params).await,
            Sqlite(pool) => pool.execute(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as `Vec<Map>`.
    pub async fn query<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Vec<Map>, BoxError> {
        match &self.pool {
            Mssql(pool) => pool.query::<N>(sql, params).await,
            MySql(pool) => pool.query::<N>(sql, params).await,
            Postgres(pool) => pool.query::<N>(sql, params).await,
            Sqlite(pool) => pool.query::<N>(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as `Vec<T>`.
    pub async fn query_as<T: DeserializeOwned, const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Vec<T>, BoxError> {
        let data = self.query::<N>(sql, params).await?;
        serde_json::from_value(data.into()).map_err(|err| err.into())
    }

    /// Executes the query in the table, and parses it as a `Map`.
    pub async fn query_one<const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Option<Map>, BoxError> {
        match &self.pool {
            Mssql(pool) => pool.query_one::<N>(sql, params).await,
            MySql(pool) => pool.query_one::<N>(sql, params).await,
            Postgres(pool) => pool.query_one::<N>(sql, params).await,
            Sqlite(pool) => pool.query_one::<N>(sql, params).await,
        }
    }

    /// Executes the query in the table, and parses it as an instance of type `T`.
    pub async fn query_one_as<T: DeserializeOwned, const N: usize>(
        &self,
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Option<T>, BoxError> {
        match self.query_one::<N>(sql, params).await? {
            Some(data) => serde_json::from_value(data.into()).map_err(|err| err.into()),
            None => Ok(None),
        }
    }
}
