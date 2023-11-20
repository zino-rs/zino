use crate::{SharedString, Uuid};
use std::time::Instant;

/// Data associated with a query.
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// Start time.
    start_time: Instant,
    /// Query ID.
    query_id: Uuid,
    /// A query.
    query: String,
    /// Arguments.
    arguments: Vec<String>,
    /// Last insert ID.
    last_insert_id: Option<i64>,
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
            query_id: Uuid::now_v7(),
            query: String::new(),
            arguments: Vec::new(),
            last_insert_id: None,
            rows_affected: None,
            success: false,
        }
    }

    /// Sets the query.
    #[inline]
    pub fn set_query(&mut self, query: impl ToString) {
        self.query = query.to_string();
    }

    /// Adds an argument to the list of query arguments.
    #[inline]
    pub fn add_argument(&mut self, value: impl ToString) {
        self.arguments.push(value.to_string());
    }

    /// Appends the query arguments.
    #[inline]
    pub fn append_arguments(&mut self, arguments: &mut Vec<String>) {
        self.arguments.append(arguments);
    }

    /// Sets the last insert ID.
    #[inline]
    pub fn set_last_insert_id(&mut self, last_insert_id: i64) {
        self.last_insert_id = Some(last_insert_id);
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

    /// Returns the query ID.
    #[inline]
    pub fn query_id(&self) -> Uuid {
        self.query_id
    }

    /// Returns the query.
    #[inline]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the query arguments.
    #[inline]
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// Returns the last insert ID.
    #[inline]
    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
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

    /// Formats the query arguments.
    #[inline]
    pub fn format_arguments(&self) -> String {
        self.arguments().join(", ")
    }

    /// Records an error message for the query.
    #[inline]
    pub fn record_error(&self, message: impl AsRef<str>) {
        let query_id = self.query_id().to_string();
        let query = self.query();
        let arguments = self.format_arguments();
        tracing::error!(query_id, query, arguments, message = message.as_ref());
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
