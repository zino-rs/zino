use self::DataSourcePool::*;
use crate::connector::Connector;
use sqlx::{mssql::MssqlPool, mysql::MySqlPool, postgres::PgPool, sqlite::SqlitePool, Error};
use toml::Table;

/// Supported data source pool.
#[derive(Debug)]
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
    ) -> Result<Self, Error> {
        match database_type {
            "mssql" => Ok(MssqlPool::new_data_source(config)),
            "mysql" => Ok(MySqlPool::new_data_source(config)),
            "postgres" => Ok(PgPool::new_data_source(config)),
            "sqlite" => Ok(SqlitePool::new_data_source(config)),
            _ => Err(Error::Protocol(format!(
                "database type `{database_type}` is unsupported"
            ))),
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
    ) -> Result<u64, Error> {
        match &self.pool {
            Mssql(pool) => pool.execute(sql, params).await,
            MySql(pool) => pool.execute(sql, params).await,
            Postgres(pool) => pool.execute(sql, params).await,
            Sqlite(pool) => pool.execute(sql, params).await,
        }
    }
}
