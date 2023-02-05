//! Database connectors.

use sqlx::{Database, Error, Pool};

/// A database connector is a pool of underlying connections.
pub trait Connector {
    /// Connection pool.
    type Pool;

    /// Creates a new connection pool with the given data source name,
    /// and immediately establish one connection.
    async fn connect(dsn: &str) -> Result<Self::Pool, Error>;

    /// Create a new connection pool with the given  data source name.
    /// The pool will establish connections only as needed.
    fn connect_lazy(dsn: &str) -> Result<Self::Pool, Error>;
}

impl<DB: Database> Connector for Pool<DB> {
    type Pool = Self;

    #[inline]
    async fn connect(dsn: &str) -> Result<Self, Error> {
        Self::connect(dsn).await
    }

    #[inline]
    fn connect_lazy(dsn: &str) -> Result<Self, Error> {
        Self::connect_lazy(dsn)
    }
}
