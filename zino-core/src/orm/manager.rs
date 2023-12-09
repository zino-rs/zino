use super::{pool::ConnectionPool, DatabasePool};
use crate::extension::TomlTableExt;
use std::time::Duration;
use toml::value::Table;

/// A manager of the connection pool.
pub trait PoolManager {
    /// Connects lazily to the database according to the config.
    fn with_config(config: &'static Table) -> Self;

    /// Checks the availability of the connection pool.
    async fn check_availability(&self) -> bool;

    /// Shuts down the connection pool.
    async fn close(&self);
}

#[cfg(feature = "orm-sqlx")]
impl PoolManager for ConnectionPool<DatabasePool> {
    fn with_config(config: &'static Table) -> Self {
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
            .unwrap_or_else(|| Duration::from_secs(24 * 60 * 60));
        let idle_timeout = config
            .get_duration("idle-timeout")
            .unwrap_or_else(|| Duration::from_secs(60 * 60));
        let acquire_timeout = config
            .get_duration("acquire-timeout")
            .unwrap_or_else(|| Duration::from_secs(60));
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
                        && let Some(cp) = super::GlobalPool::get(name)
                    {
                        if let Err(err) = conn.ping().await {
                            let name = cp.name();
                            cp.store_availability(false);
                            tracing::error!(
                                "fail to ping the database for the `{name}` service: {err}"
                            );
                            return Err(err);
                        } else {
                            cp.store_availability(true);
                        }
                    }
                    Ok(true)
                })
            })
            .connect_lazy_with(connect_options);
        Self::new(name, database, pool)
    }

    async fn check_availability(&self) -> bool {
        if let Err(err) = self.pool().acquire().await {
            let name = self.name();
            tracing::error!("fail to acquire a connection for the `{name}` service: {err}");
            self.store_availability(false);
            false
        } else {
            self.store_availability(true);
            true
        }
    }

    async fn close(&self) {
        let name = self.name();
        tracing::warn!("closing the connection pool for the `{name}` service");
        self.pool().close().await;
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
