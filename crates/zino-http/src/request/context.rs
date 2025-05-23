use std::time::Instant;
use zino_core::Uuid;

#[cfg(feature = "i18n")]
use unic_langid::LanguageIdentifier;

/// Data associated with a request-response lifecycle.
#[derive(Debug, Clone)]
pub struct Context {
    /// Start time.
    start_time: Instant,
    /// Instance.
    instance: String,
    /// Request ID.
    request_id: Uuid,
    /// Trace ID.
    trace_id: Uuid,
    /// Session ID.
    session_id: Option<String>,
    /// Locale.
    #[cfg(feature = "i18n")]
    locale: Option<LanguageIdentifier>,
}

impl Context {
    /// Creates a new instance.
    pub fn new(request_id: Uuid) -> Self {
        Self {
            start_time: Instant::now(),
            instance: String::new(),
            request_id,
            trace_id: Uuid::nil(),
            session_id: None,
            #[cfg(feature = "i18n")]
            locale: None,
        }
    }

    /// Sets the instance.
    #[inline]
    pub fn set_instance(&mut self, instance: String) {
        self.instance = instance;
    }

    /// Sets the trace ID.
    #[inline]
    pub fn set_trace_id(&mut self, trace_id: Uuid) {
        self.trace_id = trace_id;
    }

    /// Sets the session ID.
    #[inline]
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    /// Sets the locale.
    #[cfg(feature = "i18n")]
    #[inline]
    pub fn set_locale(&mut self, locale: LanguageIdentifier) {
        self.locale = Some(locale);
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Returns the instance.
    #[inline]
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Returns the request ID.
    #[inline]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Returns the trace ID.
    #[inline]
    pub fn trace_id(&self) -> Uuid {
        self.trace_id
    }

    /// Returns the session ID.
    #[inline]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns the locale.
    #[cfg(feature = "i18n")]
    pub fn locale(&self) -> Option<&LanguageIdentifier> {
        self.locale.as_ref()
    }
}
