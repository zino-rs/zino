//! Database model schema and SQL builder.

use crate::{extend::TomlTableExt, state::State};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::{sync::LazyLock, time::Duration};
use toml::value::Table;

mod column;
mod model;
mod mutation;
mod postgres;
mod query;
mod schema;

pub use column::Column;
pub use model::Model;
pub use mutation::Mutation;
pub use query::Query;
pub use schema::Schema;

use column::ColumnExt;

/// A database connection pool.
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    /// Name.
    name: &'static str,
    /// Database.
    database: &'static str,
    /// Pool.
    pool: PgPool,
}

impl ConnectionPool {
    /// Connects lazily to the database according to the config.
    pub fn connect_lazy(application_name: &str, config: &'static Table) -> Self {
        // Connect options.
        let statement_cache_capacity = config.get_usize("statement-cache-capacity").unwrap_or(100);
        let host = config.get_str("host").unwrap_or("127.0.0.1");
        let port = config.get_u16("port").unwrap_or(5432);
        let mut connect_options = PgConnectOptions::new()
            .application_name(application_name)
            .statement_cache_capacity(statement_cache_capacity)
            .host(host)
            .port(port);
        if let Some(database) = config.get_str("database") {
            let username = config
                .get_str("username")
                .expect("the `postgres.username` field should be a str");
            let password = State::decrypt_password(config)
                .expect("the `postgres.password` field should be a str");
            connect_options = connect_options
                .database(database)
                .username(username)
                .password(password.as_ref());
        }

        // Database name.
        let database = connect_options
            .get_database()
            .unwrap_or_default()
            .to_owned()
            .leak();

        // Pool options.
        let max_connections = config.get_u32("max-connections").unwrap_or(16);
        let min_connections = config.get_u32("min-connections").unwrap_or(2);
        let max_lifetime = config.get_u64("max-lifetime").unwrap_or(60 * 60);
        let idle_timeout = config.get_u64("idle-timeout").unwrap_or(10 * 60);
        let acquire_timeout = config.get_u64("acquire-timeout").unwrap_or(30);
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(Duration::from_secs(max_lifetime))
            .idle_timeout(Duration::from_secs(idle_timeout))
            .acquire_timeout(Duration::from_secs(acquire_timeout))
            .connect_lazy_with(connect_options);

        let name = config.get_str("name").unwrap_or("main");
        Self {
            name,
            database,
            pool,
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

    /// Returns a reference to the pool.
    #[inline]
    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// A list of database connection pools.
#[derive(Debug)]
struct ConnectionPools(Vec<ConnectionPool>);

impl ConnectionPools {
    /// Returns a connection pool with the specific name.
    #[inline]
    pub(crate) fn get_pool(&self, name: &str) -> Option<&ConnectionPool> {
        self.0.iter().find(|c| c.name() == name)
    }
}

/// Shared connection pools.
static SHARED_CONNECTION_POOLS: LazyLock<ConnectionPools> = LazyLock::new(|| {
    let config = State::shared().config();

    // Application name.
    let application_name = config
        .get_str("name")
        .expect("the `name` field should be a str");

    // Database connection pools.
    let mut pools = Vec::new();
    let databases = config
        .get_array("postgres")
        .expect("the `postgres` field should be an array of tables");
    for database in databases {
        if database.is_table() {
            let postgres = database
                .as_table()
                .expect("the `postgres` field should be a table");
            let pool = ConnectionPool::connect_lazy(application_name, postgres);
            pools.push(pool);
        }
    }
    ConnectionPools(pools)
});

/// Database namespace prefix.
static NAMESPACE_PREFIX: LazyLock<&'static str> = LazyLock::new(|| {
    State::shared()
        .config()
        .get_table("database")
        .expect("the `database` field should be a table")
        .get_str("namespace")
        .expect("the `database.namespace` field should be a str")
});
