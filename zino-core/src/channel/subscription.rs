use serde::{Deserialize, Serialize};

/// Subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Subscription {
    /// Session ID.
    session_id: Option<String>,
    /// Source.
    source: Option<String>,
    /// Topic.
    topic: Option<String>,
}

impl Subscription {
    /// Creates a new instance.
    #[inline]
    pub fn new(source: Option<String>, topic: Option<String>) -> Self {
        Self {
            session_id: None,
            source,
            topic,
        }
    }

    /// Sets the session ID.
    #[inline]
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    /// Sets the source.
    #[inline]
    pub fn set_source(&mut self, source: Option<String>) {
        self.source = source;
    }

    /// Sets the topic.
    #[inline]
    pub fn set_topic(&mut self, topic: Option<String>) {
        self.topic = topic;
    }

    /// Returns the session ID.
    #[inline]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns the source.
    #[inline]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Returns the topic.
    #[inline]
    pub fn topic(&self) -> Option<&str> {
        self.topic.as_deref()
    }
}
