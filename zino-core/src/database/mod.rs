//! Connection pool and ORM.

use crate::{crypto, state::SHARED_STATE};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{sync::LazyLock, time::Duration};
use toml::value::Table;

mod column;
mod model;
mod mutation;
mod query;
mod schema;

pub use column::Column;
pub use model::Model;
pub use mutation::Mutation;
pub use query::Query;
pub use schema::Schema;

/// A database connection pool.
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    /// Name.
    name: String,
    /// Database.
    database: String,
    /// Pool.
    pool: PgPool,
}

impl ConnectionPool {
    /// Encrypts the database password in the config.
    pub fn encrypt_password(config: &Table) -> Option<String> {
        let username = config
            .get("username")
            .expect("the `postgres.username` field should be specified")
            .as_str()
            .expect("the `postgres.username` field should be a str");
        let database = config
            .get("database")
            .expect("the `postgres.database` field should be specified")
            .as_str()
            .expect("the `postgres.database` field should be a str");
        let password = config
            .get("password")
            .expect("the `postgres.password` field should be specified")
            .as_str()
            .expect("the `postgres.password` field should be a str");
        let key = format!("{username}@{database}");
        crypto::encrypt(key.as_bytes(), password.as_bytes())
            .ok()
            .map(base64::encode)
    }

    /// Connects lazily to the database according to the config.
    pub fn connect_lazy(application_name: &str, config: &Table) -> Self {
        // Connect options.
        let statement_cache_capacity = config
            .get("statement-cache-capacity")
            .and_then(|t| t.as_integer())
            .and_then(|t| usize::try_from(t).ok())
            .unwrap_or(100);
        let host = config
            .get("host")
            .and_then(|t| t.as_str())
            .unwrap_or("127.0.0.1");
        let port = config
            .get("port")
            .and_then(|t| t.as_integer())
            .and_then(|t| u16::try_from(t).ok())
            .unwrap_or(5432);
        let mut connect_options = PgConnectOptions::new()
            .application_name(application_name)
            .statement_cache_capacity(statement_cache_capacity)
            .host(host)
            .port(port);
        if let Some(database) = config.get("database").and_then(|t| t.as_str()) {
            let username = config
                .get("username")
                .expect("the `postgres.username` field should be specified")
                .as_str()
                .expect("the `postgres.username` field should be a str");
            let mut password = config
                .get("password")
                .expect("the `postgres.password` field should be specified")
                .as_str()
                .expect("the `postgres.password` field should be a str");
            if let Ok(data) = base64::decode(password) {
                let key = format!("{username}@{database}");
                if let Ok(plaintext) = crypto::decrypt(key.as_bytes(), &data) {
                    password = plaintext.leak();
                }
            }
            connect_options = connect_options
                .database(database)
                .username(username)
                .password(password);
        }

        // Database name.
        let database = connect_options
            .get_database()
            .unwrap_or_default()
            .to_string();

        // Pool options.
        let max_connections = config
            .get("max-connections")
            .and_then(|t| t.as_integer())
            .and_then(|t| u32::try_from(t).ok())
            .unwrap_or(16);
        let min_connections = config
            .get("min-connections")
            .and_then(|t| t.as_integer())
            .and_then(|t| u32::try_from(t).ok())
            .unwrap_or(2);
        let max_lifetime = config
            .get("max-lifetime")
            .and_then(|t| t.as_integer().and_then(|i| i.try_into().ok()))
            .unwrap_or(60 * 60);
        let idle_timeout = config
            .get("idle-timeout")
            .and_then(|t| t.as_integer().and_then(|i| i.try_into().ok()))
            .unwrap_or(10 * 60);
        let acquire_timeout = config
            .get("acquire-timeout")
            .and_then(|t| t.as_integer().and_then(|i| i.try_into().ok()))
            .unwrap_or(30);
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(Duration::from_secs(max_lifetime))
            .idle_timeout(Duration::from_secs(idle_timeout))
            .acquire_timeout(Duration::from_secs(acquire_timeout))
            .connect_lazy_with(connect_options);

        let name = config
            .get("name")
            .and_then(|t| t.as_str())
            .unwrap_or("main")
            .to_string();
        Self {
            name,
            database,
            pool,
        }
    }

    /// Returns the name as a str.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the database as a str.
    #[inline]
    pub fn database(&self) -> &str {
        &self.database
    }

    /// Returns a reference to the pool.
    #[inline]
    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// A list of database connection pools.
#[derive(Debug, Clone)]
pub(crate) struct ConnectionPools(Vec<ConnectionPool>);

impl ConnectionPools {
    /// Returns a connection pool with the specific name.
    #[inline]
    pub(crate) fn get_pool(&self, name: &str) -> Option<&ConnectionPool> {
        self.0.iter().find(|c| c.name() == name)
    }
}

/// Shared connection pools.
pub(super) static SHARED_CONNECTION_POOLS: LazyLock<ConnectionPools> = LazyLock::new(|| {
    let config = SHARED_STATE.config();

    // Application name.
    let application_name = config
        .get("name")
        .and_then(|t| t.as_str())
        .expect("the `name` field should be specified");

    // Database connection pools.
    let mut pools = Vec::new();
    let databases = config
        .get("postgres")
        .expect("the `postgres` field should be specified")
        .as_array()
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
pub(super) static NAMESPACE_PREFIX: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_STATE
        .config()
        .get("database")
        .expect("the `database` field should be specified")
        .as_table()
        .expect("the `database` field should be a table")
        .get("namespace")
        .expect("the `database.namespace` field should be specified")
        .as_str()
        .expect("the `database.namespace` field should be a str")
});
