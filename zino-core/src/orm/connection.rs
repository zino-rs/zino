use super::{pool::ConnectionPool, DatabasePool};
use crate::extension::TomlTableExt;
use std::time::Duration;
use toml::value::Table;

/// An interface for creating a connection pool.
pub trait Connection {
    /// Connects lazily to the database according to the config.
    fn connect_with_config(config: &'static Table) -> ConnectionPool<Self>
    where
        Self: Sized;
}

#[cfg(feature = "orm-sqlx")]
impl Connection for DatabasePool {
    fn connect_with_config(config: &'static Table) -> ConnectionPool<Self> {
        use sqlx::{pool::PoolOptions, Connection};

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
        let min_connections = config.get_u32("min-connections").unwrap_or(1);
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
        let pool = PoolOptions::<super::DatabaseDriver>::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(max_lifetime)
            .idle_timeout(idle_timeout)
            .acquire_timeout(acquire_timeout)
            .test_before_acquire(false)
            .before_acquire(move |conn, meta| {
                Box::pin(async move {
                    if meta.idle_for.as_secs() > health_check_interval
                        && let Some(cp) = super::SHARED_CONNECTION_POOLS.get_pool(name)
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
        ConnectionPool::new(name, database, pool)
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "orm-mariadb", feature = "orm-mysql", feature = "orm-tidb"))] {
        use crate::state::State;
        use sqlx::mysql::MySqlConnectOptions;

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
        use crate::state::State;
        use sqlx::postgres::PgConnectOptions;

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
        use sqlx::sqlite::SqliteConnectOptions;

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
