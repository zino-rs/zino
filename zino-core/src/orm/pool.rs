use super::DatabasePool;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// A database connection pool with metadata.
#[derive(Debug)]
pub struct ConnectionPool<P = DatabasePool> {
    /// Name.
    name: &'static str,
    /// Database.
    database: &'static str,
    /// Pool.
    pool: P,
    /// Availability.
    available: AtomicBool,
}

impl<P> ConnectionPool<P> {
    /// Creates a new instance.
    #[inline]
    pub fn new(name: &'static str, database: &'static str, pool: P) -> Self {
        Self {
            name,
            database,
            pool,
            available: AtomicBool::new(true),
        }
    }

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
    pub fn pool(&self) -> &P {
        &self.pool
    }
}
