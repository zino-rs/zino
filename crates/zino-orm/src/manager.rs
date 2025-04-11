use super::{DatabasePool, pool::ConnectionPool};
use std::time::Duration;
use toml::value::Table;
use zino_core::extension::TomlTableExt;

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
        use sqlx::{Connection, Executor, pool::PoolOptions};

        let name = config.get_str("name").unwrap_or("main");

        // Connect options.
        let database = config
            .get_str("database")
            .expect("field `database` should be a str");
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
                    if meta.idle_for.as_secs() > health_check_interval {
                        if let Some(cp) = super::GlobalPool::get(name) {
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
                    }
                    Ok(true)
                })
            })
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    if let Some(time_zone) = super::TIME_ZONE.get() {
                        if cfg!(any(
                            feature = "orm-mariadb",
                            feature = "orm-mysql",
                            feature = "orm-tidb"
                        )) {
                            let sql = format!("SET time_zone = '{time_zone}';");
                            conn.execute(sql.as_str()).await?;
                        } else if cfg!(feature = "orm-postgres") {
                            let sql = format!("SET TIME ZONE '{time_zone}';");
                            conn.execute(sql.as_str()).await?;
                        }
                    }
                    Ok(())
                })
            })
            .connect_lazy_with(connect_options);
        let connection_pool = Self::new(name, database, pool);
        if config.get_bool("auto-migration").is_some_and(|b| !b) {
            connection_pool.disable_auto_migration();
        }
        connection_pool
    }

    async fn check_availability(&self) -> bool {
        let name = self.name();
        if let Err(err) = self.pool().acquire().await {
            tracing::error!("fail to acquire a connection for the `{name}` service: {err}");
            self.store_availability(false);
            false
        } else {
            tracing::info!("acquire a connection for the `{name}` service sucessfully");
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
        use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
        use zino_core::state::State;

        /// Options and flags which can be used to configure a MySQL connection.
        fn new_connect_options(database: &'static str, config: &'static Table) -> MySqlConnectOptions {
            let username = config
                .get_str("username")
                .expect("field `username` should be a str");
            let password =
                State::decrypt_password(config).expect("field `password` should be a str");

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
            if let Some(ssl_mode) = config.get_str("ssl-mode").and_then(|s| s.parse().ok()) {
                connect_options = connect_options.ssl_mode(ssl_mode);
            } else {
                connect_options = connect_options.ssl_mode(MySqlSslMode::Disabled);
            }
            connect_options
        }
    } else if #[cfg(feature = "orm-postgres")] {
        use sqlx::postgres::{PgConnectOptions, PgSslMode};
        use zino_core::state::State;

        /// Options and flags which can be used to configure a PostgreSQL connection.
        fn new_connect_options(database: &'static str, config: &'static Table) -> PgConnectOptions {
            let username = config
                .get_str("username")
                .expect("field `username` should be a str");
            let password =
                State::decrypt_password(config).expect("field `password` should be a str");

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
            if let Some(ssl_mode) = config.get_str("ssl-mode").and_then(|s| s.parse().ok()) {
                connect_options = connect_options.ssl_mode(ssl_mode);
            } else {
                connect_options = connect_options.ssl_mode(PgSslMode::Disable);
            }
            connect_options
        }
    } else {
        use sqlx::sqlite::SqliteConnectOptions;
        use zino_core::application::{Agent, Application};

        /// Options and flags which can be used to configure a SQLite connection.
        fn new_connect_options(database: &'static str, config: &'static Table) -> SqliteConnectOptions {
            let mut connect_options = SqliteConnectOptions::new().create_if_missing(true);
            if let Some(read_only) = config.get_bool("read-only") {
                connect_options = connect_options.read_only(read_only);
            }

            let database_path = Agent::parse_path(database);
            connect_options.filename(database_path)
        }
    }
}
