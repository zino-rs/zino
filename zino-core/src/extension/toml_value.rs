use crate::{extension::TomlTableExt, JsonValue, TomlValue};
use serde_json::Number;

/// Extension trait for [`toml::Value`](toml::Value).
pub trait TomlValueExt {
    /// If the `Value` is an integer, represent it as `u8` if possible.
    /// Returns `None` otherwise.
    fn as_u8(&self) -> Option<u8>;

    /// If the `Value` is an integer, represent it as `u16` if possible.
    /// Returns `None` otherwise.
    fn as_u16(&self) -> Option<u16>;

    /// If the `Value` is an integer, represent it as `u32` if possible.
    /// Returns `None` otherwise.
    fn as_u32(&self) -> Option<u32>;

    /// If the `Value` is an integer, represent it as `usize` if possible.
    /// Returns `None` otherwise.
    fn as_usize(&self) -> Option<usize>;

    /// If the `Value` is an integer, represent it as `i32` if possible.
    /// Returns `None` otherwise.
    fn as_i32(&self) -> Option<i32>;

    /// If the `Value` is a float, represent it as `f32` if possible.
    /// Returns `None` otherwise.
    fn as_f32(&self) -> Option<f32>;

    /// Converts `self` to a JSON value.
    fn to_json_value(&self) -> JsonValue;
}

impl TomlValueExt for TomlValue {
    #[inline]
    fn as_u8(&self) -> Option<u8> {
        self.as_integer().and_then(|i| u8::try_from(i).ok())
    }

    #[inline]
    fn as_u16(&self) -> Option<u16> {
        self.as_integer().and_then(|i| u16::try_from(i).ok())
    }

    #[inline]
    fn as_u32(&self) -> Option<u32> {
        self.as_integer().and_then(|i| u32::try_from(i).ok())
    }

    #[inline]
    fn as_usize(&self) -> Option<usize> {
        self.as_integer().and_then(|i| usize::try_from(i).ok())
    }

    #[inline]
    fn as_i32(&self) -> Option<i32> {
        self.as_integer().and_then(|i| i32::try_from(i).ok())
    }

    #[inline]
    fn as_f32(&self) -> Option<f32> {
        self.as_float().map(|f| f as f32)
    }

    fn to_json_value(&self) -> JsonValue {
        match self {
            TomlValue::String(s) => JsonValue::String(s.to_owned()),
            TomlValue::Integer(i) => JsonValue::Number((*i).into()),
            TomlValue::Float(f) => {
                if let Some(number) = Number::from_f64(*f) {
                    JsonValue::Number(number)
                } else {
                    JsonValue::Null
                }
            }
            TomlValue::Boolean(b) => JsonValue::Bool(*b),
            TomlValue::Datetime(dt) => JsonValue::String(dt.to_string()),
            TomlValue::Array(vec) => {
                let vec = vec.iter().map(|v| v.to_json_value()).collect();
                JsonValue::Array(vec)
            }
            TomlValue::Table(table) => JsonValue::Object(table.to_map()),
        }
    }
}
