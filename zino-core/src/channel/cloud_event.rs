use crate::{DateTime, Map};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Cloud event.
/// See [the spec](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct CloudEvent {
    /// Event id.
    id: String,
    /// Event source.
    source: String,
    /// Event topic.
    #[serde(rename = "type")]
    topic: String,
    /// Response data.
    #[serde(skip_serializing_if = "Value::is_null")]
    data: Value,
    /// Session ID.
    #[serde(rename = "sessionid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    /// Timestamp.
    #[serde(rename = "time")]
    timestamp: DateTime,
    /// Version.
    #[serde(rename = "specversion")]
    version: &'static str,
}

impl CloudEvent {
    /// Creates a new instance.
    #[inline]
    pub fn new(id: String, source: String, topic: String, data: Value) -> Self {
        Self {
            id,
            source,
            topic,
            data,
            session_id: None,
            timestamp: DateTime::now(),
            version: "1.0",
        }
    }

    /// Sets the session ID.
    #[inline]
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Returns the session ID.
    #[inline]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns the event id as a `str`.
    #[inline]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the event source as a `str`.
    #[inline]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the event topic (a.k.a *event type*) as a `str`.
    #[inline]
    pub fn topic(&self) -> &str {
        self.topic.as_str()
    }

    /// Stringifies the event data as `String`.
    #[inline]
    pub fn stringify_data(&self) -> String {
        self.data.to_string()
    }

    /// Consumes the event and returns as a json object. Panics if it fails.
    #[must_use]
    pub fn into_map(self) -> Map {
        if let Value::Object(map) = serde_json::to_value(self).unwrap() {
            map
        } else {
            panic!("the cloud event cann't be converted to a json object");
        }
    }
}
