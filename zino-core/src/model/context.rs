use crate::{SharedString, Uuid};
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

    /// Records an error message for the query.
    #[inline]
    pub fn record_error(&self, message: impl AsRef<str>) {
        let query = self.query();
        let query_id = self.query_id().to_string();
        tracing::error!(query, query_id, message = message.as_ref());
    }

    /// Emits the metrics for the query.
    #[inline]
    pub fn emit_metrics(&self, action: impl Into<SharedString>) {
        metrics::histogram!(
            "zino_model_query_duration_seconds",
            self.start_time().elapsed().as_secs_f64(),
            "action" => action.into(),
        );
    }
}

impl Default for QueryContext {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
