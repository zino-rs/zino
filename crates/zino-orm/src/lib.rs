#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]

use smallvec::SmallVec;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
};
use zino_core::{LazyLock, extension::TomlTableExt, state::State};

mod accessor;
mod aggregate;
mod column;
mod entity;
mod executor;
mod helper;
mod join;
mod manager;
mod mutation;
mod pool;
mod primary_key;
mod query;
mod row;
mod schema;
mod transaction;
mod value;
mod window;

pub use accessor::ModelAccessor;
pub use aggregate::Aggregation;
pub use column::EncodeColumn;
pub use entity::{DerivedColumn, Entity, ModelColumn};
pub use executor::Executor;
pub use helper::ModelHelper;
pub use join::JoinOn;
pub use manager::PoolManager;
pub use mutation::MutationBuilder;
pub use pool::ConnectionPool;
pub use primary_key::PrimaryKey;
pub use query::QueryBuilder;
pub use row::DecodeRow;
pub use schema::Schema;
pub use transaction::Transaction;
pub use value::IntoSqlValue;
pub use window::Window;

#[cfg(feature = "orm-sqlx")]
mod decode;
#[cfg(feature = "orm-sqlx")]
mod scalar;

#[cfg(feature = "orm-sqlx")]
pub use decode::{decode, decode_array, decode_decimal, decode_optional, decode_uuid};
#[cfg(feature = "orm-sqlx")]
pub use scalar::ScalarQuery;

cfg_if::cfg_if! {
    if #[cfg(any(feature = "orm-mariadb", feature = "orm-mysql", feature = "orm-tidb"))] {
        mod mysql;

        /// Driver name.
        static DRIVER_NAME: &str = if cfg!(feature = "orm-mariadb") {
            "mariadb"
        } else if cfg!(feature = "orm-tidb") {
            "tidb"
        } else {
            "mysql"
        };

        /// MySQL database driver.
        pub type DatabaseDriver = sqlx::MySql;

        /// MySQL database pool.
        pub type DatabasePool = sqlx::MySqlPool;

        /// MySQL database connection.
        pub type DatabaseConnection = sqlx::MySqlConnection;

        /// A single row from the MySQL database.
        pub type DatabaseRow = sqlx::mysql::MySqlRow;
    } else if #[cfg(feature = "orm-postgres")] {
        mod postgres;

        /// Driver name.
        static DRIVER_NAME: &str = "postgres";

        /// PostgreSQL database driver.
        pub type DatabaseDriver = sqlx::Postgres;

        /// PostgreSQL database pool.
        pub type DatabasePool = sqlx::PgPool;

        /// PostgreSQL database connection.
        pub type DatabaseConnection = sqlx::PgConnection;

        /// A single row from the PostgreSQL database.
        pub type DatabaseRow = sqlx::postgres::PgRow;
    } else {
        mod sqlite;

        /// Driver name.
        static DRIVER_NAME: &str = "sqlite";

        /// SQLite database driver.
        pub type DatabaseDriver = sqlx::Sqlite;

        /// SQLite database pool.
        pub type DatabasePool = sqlx::SqlitePool;

        /// SQLite database connection.
        pub type DatabaseConnection = sqlx::SqliteConnection;

        /// A single row from the SQLite database.
        pub type DatabaseRow = sqlx::sqlite::SqliteRow;
    }
}

/// A list of database connection pools.
#[derive(Debug)]
struct ConnectionPools(SmallVec<[ConnectionPool; 4]>);

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

/// Global access to the shared connection pools.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalPool;

impl GlobalPool {
    /// Gets the connection pool for the specific service.
    #[inline]
    pub fn get(name: &str) -> Option<&'static ConnectionPool> {
        SHARED_CONNECTION_POOLS.get_pool(name)
    }

    /// Iterates over the shared connection pools and
    /// attempts to establish a database connection for each of them.
    #[inline]
    pub async fn connect_all() {
        for cp in SHARED_CONNECTION_POOLS.0.iter() {
            cp.check_availability().await;
        }
    }

    /// Shuts down the shared connection pools to ensure all connections are gracefully closed.
    #[inline]
    pub async fn close_all() {
        for cp in SHARED_CONNECTION_POOLS.0.iter() {
            cp.close().await;
        }
    }
}

/// Shared connection pools.
static SHARED_CONNECTION_POOLS: LazyLock<ConnectionPools> = LazyLock::new(|| {
    let config = State::shared().config();
    let mut database_type = DRIVER_NAME;
    let mut disable_auto_migration = false;
    if let Some(database) = config.get_table("database") {
        if let Some(driver) = database.get_str("type") {
            database_type = driver;
        }
        if let Some(time_zone) = database.get_str("time-zone") {
            TIME_ZONE
                .set(time_zone)
                .expect("fail to set time zone for the database session");
        }
        if let Some(max_rows) = database.get_usize("max-rows") {
            MAX_ROWS.store(max_rows, Relaxed);
        }
        if let Some(auto_migration) = database.get_bool("auto-migration") {
            disable_auto_migration = !auto_migration;
        }
        if let Some(debug_only) = database.get_bool("debug-only") {
            DEBUG_ONLY.store(debug_only, Relaxed);
        }
    }

    // Database connection pools.
    let databases = config.get_array(database_type).unwrap_or_else(|| {
        panic!(
            "field `{database_type}` should be an array of tables; \
                please use `[[{database_type}]]` to configure a list of database services"
        )
    });
    let pools = databases
        .iter()
        .filter_map(|v| v.as_table())
        .map(|config| {
            let connection_pool = ConnectionPool::with_config(config);
            if disable_auto_migration {
                connection_pool.disable_auto_migration();
            }
            connection_pool
        })
        .collect();
    let driver = DRIVER_NAME;
    if database_type == driver {
        tracing::warn!(driver, "connect to database services lazily");
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
    State::shared()
        .get_config("database")
        .and_then(|config| {
            config
                .get_str("namespace")
                .filter(|s| !s.is_empty())
                .map(|s| [s, ":"].concat().leak())
        })
        .unwrap_or_default()
});

/// Database table prefix.
static TABLE_PREFIX: LazyLock<&'static str> = LazyLock::new(|| {
    State::shared()
        .get_config("database")
        .and_then(|config| {
            config
                .get_str("namespace")
                .filter(|s| !s.is_empty())
                .map(|s| [s, "_"].concat().leak())
        })
        .unwrap_or_default()
});

/// Optional time zone.
static TIME_ZONE: OnceLock<&'static str> = OnceLock::new();

/// Max number of returning rows.
static MAX_ROWS: AtomicUsize = AtomicUsize::new(10000);

/// Debug-only mode.
static DEBUG_ONLY: AtomicBool = AtomicBool::new(false);
