use super::DatabasePool;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

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
    /// Missed count.
    missed_count: AtomicUsize,
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
            missed_count: AtomicUsize::new(0),
        }
    }

    /// Returns `true` if the connection pool is available.
    #[inline]
    pub fn is_available(&self) -> bool {
        self.available.load(Relaxed)
    }

    /// Stores the value into the availability of the connection pool.
    pub fn store_availability(&self, available: bool) {
        self.available.store(available, Relaxed);
        if available {
            self.reset_missed_count();
        } else {
            self.increment_missed_count();
        }
    }

    /// Returns the number of missed count.
    #[inline]
    pub fn missed_count(&self) -> usize {
        self.missed_count.load(Relaxed)
    }

    /// Increments the missed count by 1.
    #[inline]
    pub fn increment_missed_count(&self) {
        self.missed_count.fetch_add(1, Relaxed);
    }

    /// Resets the number of missed count.
    #[inline]
    pub fn reset_missed_count(&self) {
        self.missed_count.store(0, Relaxed);
    }

    /// Returns `true` if the connection pool is retryable to connect.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        let missed_count = self.missed_count();
        missed_count > 2 && missed_count.is_power_of_two()
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
