//! Database schema and ORM.
//!
//! # Supported database drivers
//!
//! The following optional features are available:
//!
//! | Feature flag   | Description                                          | Default? |
//! |----------------|------------------------------------------------------|----------|
//! | `orm-mysql`    | Enables the MySQL database driver.                   | No       |
//! | `orm-postgres` | Enables the PostgreSQL database driver.              | No       |
//! | `orm-sqlite`   | Enables the SQLite database driver.                  | No       |

use crate::{extension::TomlTableExt, state::State};
use convert_case::{Case, Casing};
use sqlx::{
    pool::{Pool, PoolOptions},
    Connection,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
        LazyLock,
    },
    time::Duration,
};
use toml::value::Table;

mod accessor;
mod decode;
mod helper;
mod mutation;
mod query;
mod schema;

pub use accessor::ModelAccessor;
pub use decode::decode;
pub use helper::ModelHelper;
pub use schema::Schema;

cfg_if::cfg_if! {
    if #[cfg(feature = "orm-mysql")] {
        use sqlx::mysql::{MySql, MySqlConnectOptions, MySqlRow};

        mod mysql;

        /// Driver name.
        static DRIVER_NAME: &str = "mysql";

        /// MySQL database driver.
        pub type DatabaseDriver = MySql;

        /// A single row from the MySQL database.
        pub type DatabaseRow = MySqlRow;

        /// Options and flags which can be used to configure a MySQL connection.
        fn new_connect_options(database: &'static str, config: &'static Table) -> MySqlConnectOptions {
            let username = config
                .get_str("username")
                .expect("the `username` field should be a str");
            let password =
                State::decrypt_password(config).expect("the `password` field should be a str");

            let mut connect_options = MySqlConnectOptions::new()
                .database(database)
                .username(username)
                .password(password.as_ref());
            if let Some(host) = config.get_str("host") {
                connect_options = connect_options.host(host);
            }
            if let Some(port) = config.get_u16("port") {
                connect_options = connect_options.port(port);
            }
            connect_options
        }
    } else if #[cfg(feature = "orm-postgres")] {
        use sqlx::postgres::{PgConnectOptions, PgRow, Postgres};

        mod postgres;

        /// Driver name.
        static DRIVER_NAME: &str = "postgres";

        /// PostgreSQL database driver.
        pub type DatabaseDriver = Postgres;

        /// A single row from the PostgreSQL database.
        pub type DatabaseRow = PgRow;

        /// Options and flags which can be used to configure a PostgreSQL connection.
        fn new_connect_options(database: &'static str, config: &'static Table) -> PgConnectOptions {
            let username = config
                .get_str("username")
                .expect("the `username` field should be a str");
            let password =
                State::decrypt_password(config).expect("the `password` field should be a str");

            let mut connect_options = PgConnectOptions::new()
                .database(database)
                .username(username)
                .password(password.as_ref());
            if let Some(host) = config.get_str("host") {
                connect_options = connect_options.host(host);
            }
            if let Some(port) = config.get_u16("port") {
                connect_options = connect_options.port(port);
            }
            connect_options
        }
    } else {
        use sqlx::sqlite::{Sqlite, SqliteConnectOptions, SqliteRow};

        mod sqlite;

        /// Driver name.
        static DRIVER_NAME: &str = "sqlite";

        /// SQLite database driver.
        pub type DatabaseDriver = Sqlite;

        /// A single row from the SQLite database.
        pub type DatabaseRow = SqliteRow;

        /// Options and flags which can be used to configure a SQLite connection.
        fn new_connect_options(database: &'static str, config: &'static Table) -> SqliteConnectOptions {
            let mut connect_options = SqliteConnectOptions::new().create_if_missing(true);
            if let Some(read_only) = config.get_bool("read_only") {
                connect_options = connect_options.read_only(read_only);
            }

            let database_path = std::path::Path::new(database);
            let database_file = if database_path.is_relative() {
                crate::application::PROJECT_DIR.join(database_path)
            } else {
                database_path.to_path_buf()
            };
            connect_options.filename(database_file)
        }
    }
}

/// A database connection pool based on [`sqlx::Pool`](sqlx::pool::Pool).
#[derive(Debug)]
pub struct ConnectionPool {
    /// Name.
    name: &'static str,
    /// Database.
    database: &'static str,
    /// Pool.
    pool: Pool<DatabaseDriver>,
    /// Availability.
    available: AtomicBool,
}

impl ConnectionPool {
    /// Returns `true` if the connection pool is available.
    #[inline]
    pub fn is_available(&self) -> bool {
        self.available.load(Relaxed)
    }

    /// Stores the value into the availability of the connection pool.
    #[inline]
    pub fn store_availability(&self, available: bool) {
        self.available.store(available, Relaxed);
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
    pub fn pool(&self) -> &Pool<DatabaseDriver> {
        &self.pool
    }

    /// Connects lazily to the database according to the config.
    pub fn connect_lazy(config: &'static Table) -> Self {
        let name = config.get_str("name").unwrap_or("main");

        // Connect options.
        let database = config
            .get_str("database")
            .expect("the `database` field should be a str");
        let mut connect_options = new_connect_options(database, config);
        if let Some(statement_cache_capacity) = config.get_usize("statement-cache-capacity") {
            connect_options = connect_options.statement_cache_capacity(statement_cache_capacity);
        }

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
        let health_check_interval = config.get_u64("health-check-interval").unwrap_or(60);
        let pool = PoolOptions::<DatabaseDriver>::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(max_lifetime)
            .idle_timeout(idle_timeout)
            .acquire_timeout(acquire_timeout)
            .test_before_acquire(false)
            .before_acquire(move |conn, meta| {
                Box::pin(async move {
                    if meta.idle_for.as_secs() > health_check_interval
                        && let Some(cp) = SHARED_CONNECTION_POOLS.get_pool(name)
                    {
                        if let Err(err) = conn.ping().await {
                            cp.store_availability(false);
                            return Err(err);
                        } else {
                            cp.store_availability(cp.pool().num_idle() > 0);
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
}

/// A list of database connection pools.
#[derive(Debug)]
struct ConnectionPools(Vec<ConnectionPool>);

impl ConnectionPools {
    /// Returns a connection pool with the specific name.
    pub(crate) fn get_pool(&self, name: &str) -> Option<&ConnectionPool> {
        let mut pool = None;
        for cp in self.0.iter().filter(|cp| cp.name() == name) {
            if cp.is_available() {
                return Some(cp);
            } else {
                pool = Some(cp);
            }
        }
        pool
    }
}

/// Shared connection pools.
static SHARED_CONNECTION_POOLS: LazyLock<ConnectionPools> = LazyLock::new(|| {
    let config = State::shared().config();
    let Some(database_config) = config.get_table("database") else {
        return ConnectionPools(Vec::new());
    };

    // Database connection pools.
    let driver = DRIVER_NAME;
    let database_type = database_config.get_str("type").unwrap_or(driver);
    let databases = config
        .get_array(database_type)
        .unwrap_or_else(|| panic!("the `{database_type}` field should be an array of tables"));
    let pools = databases
        .iter()
        .filter_map(|v| v.as_table())
        .map(ConnectionPool::connect_lazy)
        .collect();
    if database_type == driver {
        tracing::warn!(driver, "connect to the database lazily");
    } else {
        tracing::error!(
            driver,
            "invalid database type `{database_type}` for the driver `{driver}`"
        );
    }
    ConnectionPools(pools)
});

/// Database namespace prefix.
static NAMESPACE_PREFIX: LazyLock<&'static str> = LazyLock::new(|| {
    let config = State::shared()
        .get_config("database")
        .expect("the `database` field should be a table");
    let max_rows = config.get_usize("max-rows").unwrap_or(10000);
    MAX_ROWS.store(max_rows, Relaxed);
    config
        .get_str("namespace")
        .expect("the `namespace` field should be a str")
        .to_case(Case::Snake)
        .leak()
});

/// Max number of returning rows.
static MAX_ROWS: AtomicUsize = AtomicUsize::new(10000);
