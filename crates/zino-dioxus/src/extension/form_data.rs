use dioxus::events::FormData;
use std::time::Duration;
use zino_core::{
    datetime::{self, Date, DateTime, Time},
    extension::JsonObjectExt,
    model::Model,
    validation::Validation,
    Decimal, JsonValue, Map, Uuid,
};

/// Extension trait for `FormData`.
pub trait FormDataExt {
    /// Extracts the string value.
    fn get_string(&self) -> Option<String>;

    /// Extracts the string value and parses it as `Uuid`.
    /// If the `Uuid` is `nil`, it also returns `None`.
    fn parse_uuid(&self) -> Option<Result<Uuid, uuid::Error>>;

    /// Parses the JSON value as `Decimal`.
    fn parse_decimal(&self) -> Option<Result<Decimal, rust_decimal::Error>>;

    /// Parses the string value as `Date`.
    fn parse_date(&self) -> Option<Result<Date, chrono::format::ParseError>>;

    /// Parses the string value as `Time`.
    fn parse_time(&self) -> Option<Result<Time, chrono::format::ParseError>>;

    /// Parses the string value as `DateTime`.
    fn parse_date_time(&self) -> Option<Result<DateTime, chrono::format::ParseError>>;

    /// Parses the string value as `Duration`.
    fn parse_duration(&self) -> Option<Result<Duration, datetime::ParseDurationError>>;

    /// Attempts to read the map as an instance of the model `M`.
    fn read_as_model<M: Model>(&self) -> Result<M, Validation>;

    /// Converts `self` to a JSON object.
    fn to_map(&self) -> Map;
}

impl FormDataExt for FormData {
    fn get_string(&self) -> Option<String> {
        let value = self.value();
        (!value.is_empty()).then_some(value)
    }

    fn parse_uuid(&self) -> Option<Result<Uuid, uuid::Error>> {
        self.get_string()
            .filter(|s| !s.chars().all(|c| c == '0' || c == '-'))
            .map(|s| s.parse())
    }

    #[inline]
    fn parse_decimal(&self) -> Option<Result<Decimal, rust_decimal::Error>> {
        self.get_string().map(|s| s.parse())
    }

    #[inline]
    fn parse_date(&self) -> Option<Result<Date, chrono::format::ParseError>> {
        self.get_string().map(|s| s.parse())
    }

    #[inline]
    fn parse_time(&self) -> Option<Result<Time, chrono::format::ParseError>> {
        self.get_string().map(|s| s.parse())
    }

    #[inline]
    fn parse_date_time(&self) -> Option<Result<DateTime, chrono::format::ParseError>> {
        self.get_string().map(|s| s.parse())
    }

    #[inline]
    fn parse_duration(&self) -> Option<Result<Duration, datetime::ParseDurationError>> {
        self.get_string()
            .map(|s| datetime::parse_duration(s.as_str()))
    }

    fn read_as_model<M: Model>(&self) -> Result<M, Validation> {
        let mut model = M::new();
        let validation = model.read_map(&self.to_map());
        if validation.is_success() {
            Ok(model)
        } else {
            Err(validation)
        }
    }

    fn to_map(&self) -> Map {
        let values = self.values();
        let mut map = Map::with_capacity(values.len());
        for (key, value) in values.into_iter() {
            let mut vec = value.to_vec();
            if vec.len() == 1 {
                if let Some(value) = vec.pop() {
                    if value == "null" {
                        map.upsert(key, JsonValue::Null);
                    } else {
                        map.upsert(key, value);
                    }
                }
            } else {
                map.upsert(key, vec);
            }
        }
        map
    }
}
