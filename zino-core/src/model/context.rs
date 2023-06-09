use crate::Uuid;
use std::time::Instant;

/// Data associated with a query.
#[derive(Debug, Clone)]
pub struct QueryContext<'a> {
    /// Start time.
    start_time: Instant,
    /// A query.
    query: &'a str,
    /// Query ID.
    query_id: Uuid,
}

impl<'a> QueryContext<'a> {
    /// Creates a new instance.
    #[inline]
    pub fn new(query: &'a str) -> Self {
        Self {
            start_time: Instant::now(),
            query,
            query_id: Uuid::new_v4(),
        }
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Returns the query.
    #[inline]
    pub fn query(&self) -> &str {
        self.query
    }

    /// Returns the query ID.
    #[inline]
    pub fn query_id(&self) -> Uuid {
        self.query_id
    }
}
