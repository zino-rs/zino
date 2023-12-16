use crate::{datetime::DateTime, JsonValue, Map, SharedString};
use serde::{Deserialize, Serialize};

/// Cloud event.
/// See [the spec](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct CloudEvent<T = ()> {
    /// Spec version.
    #[serde(rename = "specversion")]
    spec_version: SharedString,
    /// Event ID.
    id: String,
    /// Event source.
    source: String,
    /// Event type.
    #[serde(rename = "type")]
    event_type: String,
    /// Timestamp.
    #[serde(rename = "time")]
    timestamp: DateTime,
    /// Event data.
    #[serde(skip_serializing_if = "JsonValue::is_null")]
    data: JsonValue,
    /// Optional data content type.
    #[serde(rename = "datacontenttype")]
    data_content_type: Option<SharedString>,
    /// Optional data schema.
    #[serde(rename = "dataschema")]
    data_schema: Option<SharedString>,
    /// Optional subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<SharedString>,
    /// Optional session ID.
    #[serde(rename = "sessionid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    /// Extensions.
    #[serde(flatten)]
    extensions: T,
}

impl<T: Default> CloudEvent<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(id: String, source: String, event_type: String) -> Self {
        Self {
            spec_version: "1.0".into(),
            id,
            source,
            event_type,
            timestamp: DateTime::now(),
            data: JsonValue::Null,
            data_content_type: None,
            data_schema: None,
            subject: None,
            session_id: None,
            extensions: T::default(),
        }
    }
}

impl<T> CloudEvent<T> {
    /// Sets the event data.
    #[inline]
    pub fn set_data(&mut self, data: impl Into<JsonValue>) {
        self.data = data.into();
    }

    /// Sets the subject.
    #[inline]
    pub fn set_subject(&mut self, subject: impl Into<SharedString>) {
        self.subject = Some(subject.into());
    }

    /// Sets the session ID.
    #[inline]
    pub fn set_session_id(&mut self, session_id: impl ToString) {
        self.session_id = Some(session_id.to_string());
    }

    /// Returns the event ID as a `str`.
    #[inline]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the event source as a `str`.
    #[inline]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the event type as a `str`.
    #[inline]
    pub fn event_type(&self) -> &str {
        self.event_type.as_str()
    }

    /// Returns a reference to the optional subject.
    #[inline]
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    /// Returns a reference to the optional session ID.
    #[inline]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Stringifies the event data as `String`.
    #[inline]
    pub fn stringify_data(&self) -> String {
        self.data.to_string()
    }
}

impl<T: Serialize> CloudEvent<T> {
    /// Consumes the event and returns as a json object.
    ///
    /// # Panics
    ///
    /// It will panic if the model cann't be converted to a json object.
    #[must_use]
    pub fn into_map(self) -> Map {
        match serde_json::to_value(self) {
            Ok(JsonValue::Object(map)) => map,
            _ => panic!("the cloud event cann't be converted to a json object"),
        }
    }
}
