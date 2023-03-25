//! Database schema and ORM.

use crate::{extend::TomlTableExt, state::State};
use sqlx::{
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    Connection, Database, Pool, Postgres,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock,
    },
    time::Duration,
};
use toml::value::Table;

mod mutation;
mod postgres;
mod query;
mod schema;

pub use schema::Schema;

/// A database connection pool.
#[derive(Debug)]
pub struct ConnectionPool<DB = Postgres>
where
    DB: Database,
{
    /// Name.
    name: &'static str,
    /// Database.
    database: &'static str,
    /// Pool.
    pool: Pool<DB>,
    /// Availability.
    available: AtomicBool,
}

impl ConnectionPool<Postgres> {
    /// Returns `true` if the connection pool is available.
    #[inline]
    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    /// Stores the value into the availability of the connection pool.
    #[inline]
    pub fn store_availability(&self, available: bool) {
        self.available.store(available, Ordering::Relaxed);
    }

    /// Connects lazily to the database according to the config.
    pub fn connect_lazy(application_name: &'static str, config: &'static Table) -> Self {
        let name = config.get_str("name").unwrap_or("main");

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
        let max_lifetime = config
            .get_duration("max-lifetime")
            .unwrap_or_else(|| Duration::from_secs(60 * 60));
        let idle_timeout = config
            .get_duration("idle-timeout")
            .unwrap_or_else(|| Duration::from_secs(10 * 60));
        let acquire_timeout = config
            .get_duration("acquire-timeout")
            .unwrap_or_else(|| Duration::from_secs(30));
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(max_lifetime)
            .idle_timeout(idle_timeout)
            .acquire_timeout(acquire_timeout)
            .test_before_acquire(false)
            .before_acquire(move |conn, meta| {
                Box::pin(async move {
                    if meta.idle_for.as_secs() > 60 &&
                        let Some(pool) = SHARED_CONNECTION_POOLS.get_pool(name)
                    {
                        if let Err(err) = conn.ping().await {
                            pool.store_availability(false);
                            return Err(err);
                        } else {
                            pool.store_availability(true);
                        }
                    }
                    Ok(true)
                })
            })
            .connect_lazy_with(connect_options);

        Self {
            name,
            database,
            pool,
            available: AtomicBool::new(true),
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
    pub(crate) fn get_pool(&self, name: &str) -> Option<&ConnectionPool> {
        let mut pool = None;
        for c in self.0.iter() {
            if c.name() == name {
                if c.is_available() {
                    return Some(c);
                } else {
                    pool = Some(c);
                }
            }
        }
        pool
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
    let databases = config
        .get_array("postgres")
        .expect("the `postgres` field should be an array of tables");
    let pools = databases
        .iter()
        .filter_map(|v| v.as_table())
        .map(|database| ConnectionPool::connect_lazy(application_name, database))
        .collect::<Vec<_>>();
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
