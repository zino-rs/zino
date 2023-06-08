use std::time::Instant;

/// Data associated with a query.
#[derive(Debug, Clone)]
pub struct QueryContext<'a> {
    /// A query.
    query: &'a str,
    /// Start time.
    start_time: Instant,
}

impl<'a> QueryContext<'a> {
    /// Creates a new instance.
    #[inline]
    pub fn new(query: &'a str) -> Self {
        Self {
            query,
            start_time: Instant::now(),
        }
    }

    /// Returns the query.
    #[inline]
    pub fn query(&self) -> &str {
        self.query
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}
