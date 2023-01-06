use crate::crypto;
use sqlx::{postgres::PgPoolOptions, Error, PgPool};
use toml::value::Table;

mod column;
mod model;
mod mutation;
mod query;
mod schema;

// Reexports.
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
        let user = config
            .get("user")
            .expect("the `postgres.user` field is missing")
            .as_str()
            .expect("the `postgres.user` field should be a str");
        let database = config
            .get("database")
            .expect("the `postgres.database` field is missing")
            .as_str()
            .expect("the `postgres.database` field should be a str");
        let password = config
            .get("password")
            .expect("the `postgres.password` field is missing")
            .as_str()
            .expect("the `postgres.password` field should be a str");
        let key = format!("{user}@{database}");
        crypto::encrypt(key.as_bytes(), password.as_bytes())
            .ok()
            .map(base64::encode)
    }

    /// Connects lazily to the database according to the config.
    pub fn connect_lazy(config: &Table) -> Result<Self, Error> {
        let host = config
            .get("host")
            .expect("the `postgres.host` field is missing")
            .as_str()
            .expect("the `postgres.host` field should be a str");
        let port = config
            .get("port")
            .expect("the `postgres.port` field is missing")
            .as_integer()
            .expect("the `postgres.port` field should be an integer");
        let user = config
            .get("user")
            .expect("the `postgres.user` field is missing")
            .as_str()
            .expect("the `postgres.user` field should be a str");
        let database = config
            .get("database")
            .expect("the `postgres.database` field is missing")
            .as_str()
            .expect("the `postgres.database` field should be a str");
        let mut password = config
            .get("password")
            .expect("the `postgres.password` field is missing")
            .as_str()
            .expect("the `postgres.password` field should be a str");
        if let Ok(data) = base64::decode(password) {
            let key = format!("{user}@{database}");
            if let Ok(plaintext) = crypto::decrypt(key.as_bytes(), &data) {
                password = plaintext.leak();
            }
        }

        let connection_string = format!("postgres://{user}:{password}@{host}:{port}/{database}");
        let max_connections = config
            .get("max-connections")
            .and_then(|t| t.as_integer())
            .and_then(|t| u32::try_from(t).ok())
            .unwrap_or(16);
        let min_connections = config
            .get("min-connections")
            .and_then(|t| t.as_integer())
            .and_then(|t| u32::try_from(t).ok())
            .unwrap_or(1);
        PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .connect_lazy(&connection_string)
            .map(|pool| {
                let name = config
                    .get("name")
                    .and_then(|t| t.as_str())
                    .unwrap_or("main");
                Self {
                    name: name.to_string(),
                    database: database.to_string(),
                    pool,
                }
            })
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
