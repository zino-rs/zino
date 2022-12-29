use serde::Serialize;
use std::time::Instant;
use uuid::Uuid;

/// Data associated with a request-response lifecycle.
#[derive(Debug, Clone, Serialize)]
pub struct Context {
    /// Start time.
    #[serde(skip)]
    start_time: Instant,
    /// Request path.
    request_path: String,
    /// Request ID.
    request_id: Uuid,
    /// Trace ID.
    trace_id: Uuid,
    /// Session ID.
    session_id: Option<String>,
}

impl Context {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_path: String::new(),
            request_id: Uuid::new_v4(),
            trace_id: Uuid::nil(),
            session_id: None,
        }
    }

    /// Sets the request path.
    #[inline]
    pub fn set_request_path(&mut self, request_path: impl Into<String>) {
        self.request_path = request_path.into();
    }

    /// Sets the request ID.
    #[inline]
    pub fn set_request_id(&mut self, request_id: Uuid) {
        self.request_id = request_id;
    }

    /// Sets the trace ID.
    #[inline]
    pub fn set_trace_id(&mut self, trace_id: Uuid) {
        self.trace_id = trace_id;
    }

    /// Sets the session ID.
    #[inline]
    pub fn set_session_id(&mut self, session_id: impl Into<Option<String>>) {
        self.session_id = session_id.into();
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Returns the request path.
    #[inline]
    pub fn request_path(&self) -> &str {
        &self.request_path
    }

    /// Returns the request id.
    #[inline]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Returns the trace id.
    #[inline]
    pub fn trace_id(&self) -> Uuid {
        self.trace_id
    }

    /// Returns the session ID.
    #[inline]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
