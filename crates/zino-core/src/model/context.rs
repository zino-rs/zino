use crate::Uuid;
use std::time::Instant;

/// Data associated with a query.
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// Model name.
    model_name: &'static str,
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
    /// Indicates the query execution is cancelled or not.
    cancelled: bool,
}

impl QueryContext {
    /// Creates a new instance.
    #[inline]
    pub fn new(model_name: &'static str) -> Self {
        Self {
            model_name,
            start_time: Instant::now(),
            query_id: Uuid::now_v7(),
            query: String::new(),
            arguments: Vec::new(),
            last_insert_id: None,
            rows_affected: None,
            success: false,
            cancelled: false,
        }
    }

    /// Sets the query.
    #[inline]
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
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
    pub fn set_query_result(&mut self, rows_affected: impl Into<Option<u64>>, success: bool) {
        self.rows_affected = rows_affected.into();
        self.success = success;
        self.cancelled = false;
    }

    /// Cancells the query execution.
    #[inline]
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Returns the model name.
    #[inline]
    pub fn model_name(&self) -> &'static str {
        self.model_name
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

    /// Returns `true` if the query execution is cancelled.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Returns `true` if the query execution is successful.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Formats the query arguments as a `String` if they exist.
    #[inline]
    pub fn format_arguments(&self) -> Option<String> {
        let arguments = self.arguments();
        (!arguments.is_empty()).then(|| arguments.join(", "))
    }

    /// Records an error message for the query.
    pub fn record_error(&self, message: impl AsRef<str>) {
        fn inner(ctx: &QueryContext, message: &str) {
            let model_name = ctx.model_name();
            let query_id = ctx.query_id().to_string();
            let query = ctx.query();
            let arguments = ctx.format_arguments();
            if ctx.is_cancelled() {
                tracing::warn!(
                    cancelled = true,
                    model_name,
                    query_id,
                    query,
                    arguments,
                    message,
                );
            } else {
                tracing::error!(model_name, query_id, query, arguments, message);
            }
        }
        inner(self, message.as_ref())
    }

    /// Emits the metrics for the query.
    #[cfg(feature = "metrics")]
    #[inline]
    pub fn emit_metrics(&self, action: impl Into<crate::SharedString>) {
        fn inner(ctx: &QueryContext, action: crate::SharedString) {
            metrics::histogram!(
                "zino_model_query_duration_seconds",
                "model_name" => ctx.model_name(),
                "action" => action,
            )
            .record(ctx.start_time().elapsed().as_secs_f64());
        }
        inner(self, action.into())
    }
}
