//! Database schema and ORM.
//!
//! # Supported database drivers
//!
//! The following optional features are available:
//!
//! | Feature flag   | Description                                          | Default? |
//! |----------------|------------------------------------------------------|----------|
//! | `orm-mariadb`  | Enables the MariaDB database driver.                 | No       |
//! | `orm-mysql`    | Enables the MySQL database driver.                   | No       |
//! | `orm-postgres` | Enables the PostgreSQL database driver.              | No       |
//! | `orm-sqlite`   | Enables the SQLite database driver.                  | No       |
//! | `orm-tidb`     | Enables the TiDB database driver.                    | No       |
//!
//! # Design references
//!
//! The design of our ORM is inspired by [`Mongoose`], [`Prisma`], [`TypeORM`] and [`PostgREST`].
//!
//! ```rust,ignore
//! use zino_core::{model::{Mutation, Query}, orm::Schema};
//!
//! // Constructs a model `Query` with JSON expressions.
//! let query = Query::new(json!({
//!     "$or": [
//!         {
//!             "roles": "worker",
//!             "visibility": "Public",
//!         },
//!         {
//!             "roles": { "$in": ["admin", "auditor"] },
//!             "visibility": { "$ne": "Public" },
//!         },
//!     ],
//!     "status": { "$nin": ["Deleted", "Locked"] },
//! }));
//!
//! // Constructs a model `Mutation` with JSON expressions.
//! let mut mutation = Mutation::new(json!({
//!     "status": "Active",
//!     "refreshed_at": DateTime::now(),
//!     "$inc": { "refresh_count": 1 },
//! }));
//!
//! // Updates the models using `update_many` provided by the `Schema` trait.
//! let ctx = User::update_many(&query, &mut mutation).await?;
//! ctx.emit_metrics("user_refresh");
//!
//! // Constructs a model `Query` with projection fields.
//! let mut query = Query::new(json!({
//!     "project.start_date": { "$le": "2023-10-07" },
//!     "project.start_date": { "$ge": "2023-10-01" },
//!     "task.status": "Completed",
//! }));
//! query.allow_fields(&[
//!     "task.id",
//!     "task.name",
//!     "task.status",
//!     "task.project_id",
//!     "project.start_date",
//!     "project.end_date",
//! ]);
//! query.order_by_desc("task.updated_at");
//!
//! // Performs a LEFT OUTER JOIN using `lookup` provided by the `Schema` trait.
//! let entries = Task::lookup::<Project, Map>(&query, &[("project_id", "id")]).await?;
//!
//! // Executes the raw SQL with interpolations `${param}` and argument bindings `#{param}`.
//! let sql =
//!     "SELECT u.id, u.name, u.tags, t.id, t.name \
//!         FROM ${user_table} u INNER JOIN ${tag_table} t \
//!             ON t.id = ANY(u.tags) AND t.category = #{category};";
//! let params = json!({
//!     "user_table": User::table_name(),
//!     "tag_table": Tag::table_name(),
//!     "category": "Rustacean",
//! });
//! let records = User::query::<Record>(sql, params.as_object()).await?;
//! ```
//!
//! # Query operators
//!
//! | Name       | MySQL               | PostgreSQL       | SQLite                |
//! |------------|---------------------|------------------|-----------------------|
//! | `$and`     | `AND`               | `AND`            | `AND`                 |
//! | `$or`      | `OR`                | `OR`             | `OR`                  |
//! | `$not`     | `NOT`               | `NOT`            | `NOT`                 |
//! | `$rand`    | `rand()`            | `random()`       | `abs(random())`       |
//! | `$text`    | `match() against()` | `to_tsvector()`  | `MATCH`               |
//! | `$ovlp`    | `overlaps()`        | `OVERLAPS`       | N/A                   |
//! | `$eq`      | `=`                 | `=`              | `=`                   |
//! | `$ne`      | `<>`                | `<>`             | `<>`                  |
//! | `$lt`      | `<`                 | `<`              | `<`                   |
//! | `$le`      | `<=`                | `<=`             | `<=`                  |
//! | `$gt`      | `>`                 | `>`              | `>`                   |
//! | `$ge`      | `>=`                | `>=`             | `>=`                  |
//! | `$in`      | `IN`                | `IN`             | `IN`                  |
//! | `$nin`     | `NOT IN`            | `NOT IN`         | `NOT IN`              |
//! | `$betw`    | `BETWEEN AND`       | `BETWEEN AND`    | `BETWEEN AND`         |
//! | `$like`    | `LIKE`              | `LIKE`           | `LIKE`                |
//! | `$ilike`   | `ILIKE`             | `ILIKE`          | N/A                   |
//! | `$rlike`   | `RLIKE`             | `~*`             | `REGEXP`              |
//! | `$glob`    | N/A                 | N/A              | `GLOB`                |
//! | `$is`      | `IS`                | `IS`             | `IS`                  |
//! | `$size`    | `json_length()`     | `array_length()` | `json_array_length()` |
//!
//! [`Mongoose`]: https://mongoosejs.com/
//! [`Prisma`]: https://www.prisma.io/
//! [`TypeORM`]: https://typeorm.io/
//! [`PostgREST`]: https://postgrest.org/

use crate::{extension::TomlTableExt, state::State};
use smallvec::SmallVec;
use std::sync::{
    atomic::{AtomicUsize, Ordering::Relaxed},
    LazyLock,
};

mod accessor;
mod column;
mod executor;
mod helper;
mod manager;
mod mutation;
mod pool;
mod query;
mod schema;
mod transaction;

pub use accessor::ModelAccessor;
pub use executor::Executor;
pub use helper::ModelHelper;
pub use manager::PoolManager;
pub use pool::ConnectionPool;
pub use schema::Schema;
pub use transaction::Transaction;

#[cfg(feature = "orm-sqlx")]
mod decode;
#[cfg(feature = "orm-sqlx")]
mod scalar;

#[cfg(feature = "orm-sqlx")]
pub use decode::{decode, decode_array};
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
        pub type DatabaseDriver = sqlx::mysql::MySql;

        /// MySQL database pool.
        pub type DatabasePool = sqlx::mysql::MySqlPool;

        /// MySQL database connection.
        pub type DatabaseConnection = sqlx::MySqlConnection;

        /// A single row from the MySQL database.
        pub type DatabaseRow = sqlx::mysql::MySqlRow;
    } else if #[cfg(feature = "orm-postgres")] {
        mod postgres;

        /// Driver name.
        static DRIVER_NAME: &str = "postgres";

        /// PostgreSQL database driver.
        pub type DatabaseDriver = sqlx::postgres::Postgres;

        /// PostgreSQL database pool.
        pub type DatabasePool = sqlx::postgres::PgPool;

        /// PostgreSQL database connection.
        pub type DatabaseConnection = sqlx::PgConnection;

        /// A single row from the PostgreSQL database.
        pub type DatabaseRow = sqlx::postgres::PgRow;
    } else {
        mod sqlite;

        /// Driver name.
        static DRIVER_NAME: &str = "sqlite";

        /// SQLite database driver.
        pub type DatabaseDriver = sqlx::sqlite::Sqlite;

        /// SQLite database pool.
        pub type DatabasePool = sqlx::sqlite::SqlitePool;

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
    let Some(database_config) = config.get_table("database") else {
        return ConnectionPools(SmallVec::new());
    };

    // Database connection pools.
    let driver = DRIVER_NAME;
    let database_type = database_config.get_str("type").unwrap_or(driver);
    let databases = config.get_array(database_type).unwrap_or_else(|| {
        panic!(
            "the `{database_type}` field should be an array of tables; \
                please use `[[{database_type}]]` to configure a list of database services"
        )
    });
    let pools = databases
        .iter()
        .filter_map(|v| v.as_table())
        .map(ConnectionPool::with_config)
        .collect();
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
            if let Some(max_rows) = config.get_usize("max-rows") {
                MAX_ROWS.store(max_rows, Relaxed);
            }
            config
                .get_str("namespace")
                .filter(|s| !s.is_empty())
                .map(|s| [s, "_"].concat().leak())
        })
        .unwrap_or_default()
});

/// Max number of returning rows.
static MAX_ROWS: AtomicUsize = AtomicUsize::new(10000);
