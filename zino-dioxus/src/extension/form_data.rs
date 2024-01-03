use dioxus::events::FormData;
use std::time::Duration;
use zino_core::{
    datetime::{self, Date, DateTime, Time},
    extension::JsonObjectExt,
    Map, Uuid,
};

/// Extension trait for [`FormData`].
pub trait FormDataExt {
    /// Extracts the string value.
    fn get_str(&self) -> Option<&str>;

    /// Extracts the string value and parses it as `Uuid`.
    /// If the `Uuid` is `nil`, it also returns `None`.
    fn parse_uuid(&self) -> Option<Result<Uuid, uuid::Error>>;

    /// Parses the string value as `Date`.
    fn parse_date(&self) -> Option<Result<Date, chrono::format::ParseError>>;

    /// Parses the string value as `Time`.
    fn parse_time(&self) -> Option<Result<Time, chrono::format::ParseError>>;

    /// Parses the string value as `DateTime`.
    fn parse_datetime(&self) -> Option<Result<DateTime, chrono::format::ParseError>>;

    /// Parses the string value as `Duration`.
    fn parse_duration(&self) -> Option<Result<Duration, datetime::ParseDurationError>>;

    /// Converts `self` to a JSON object.
    fn to_map(&self) -> Map;
}

impl FormDataExt for FormData {
    fn get_str(&self) -> Option<&str> {
        let value = self.value.as_str();
        (!value.is_empty()).then_some(value)
    }

    fn parse_uuid(&self) -> Option<Result<Uuid, uuid::Error>> {
        self.get_str()
            .map(|s| s.trim_start_matches("urn:uuid:"))
            .filter(|s| !s.chars().all(|c| c == '0' || c == '-'))
            .map(|s| s.parse())
    }

    #[inline]
    fn parse_date(&self) -> Option<Result<Date, chrono::format::ParseError>> {
        self.get_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_time(&self) -> Option<Result<Time, chrono::format::ParseError>> {
        self.get_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_datetime(&self) -> Option<Result<DateTime, chrono::format::ParseError>> {
        self.get_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_duration(&self) -> Option<Result<Duration, datetime::ParseDurationError>> {
        self.get_str().map(datetime::parse_duration)
    }

    fn to_map(&self) -> Map {
        let values = &self.values;
        let mut map = Map::with_capacity(values.len());
        for (key, value) in values.iter() {
            if let Some([value]) = values.get(key).map(|v| v.as_slice()) {
                map.upsert(key, value.clone());
            } else {
                map.upsert(key, value.clone());
            }
        }
        map
    }
}
