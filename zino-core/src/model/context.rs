use crate::Uuid;
use std::time::Instant;

/// Data associated with a query.
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// Start time.
    start_time: Instant,
    /// A query.
    query: String,
    /// Query ID.
    query_id: Uuid,
    /// Number of rows affected.
    rows_affected: Option<u64>,
    /// Indicates the query execution is successful or not.
    success: bool,
}

impl QueryContext {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            query: String::new(),
            query_id: Uuid::new_v4(),
            rows_affected: None,
            success: false,
        }
    }

    /// Sets the query.
    #[inline]
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    /// Sets the query result.
    #[inline]
    pub fn set_query_result(&mut self, rows_affected: Option<u64>, success: bool) {
        self.rows_affected = rows_affected;
        self.success = success;
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Returns the query.
    #[inline]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the query ID.
    #[inline]
    pub fn query_id(&self) -> Uuid {
        self.query_id
    }

    /// Returns the number of rows affected.
    #[inline]
    pub fn rows_affected(&self) -> Option<u64> {
        self.rows_affected
    }

    /// Returns `true` if the query execution is success.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success
    }
}

impl Default for QueryContext {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
