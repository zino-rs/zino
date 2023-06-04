use crate::JsonValue;
use std::{
    borrow::Cow,
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
};

/// Extension trait for [`serde_json::Value`](serde_json::Value).
pub trait JsonValueExt {
    /// Parses the json value as `bool`.
    fn parse_bool(&self) -> Option<Result<bool, ParseBoolError>>;

    /// Parses the json value as `u8`.
    fn parse_u8(&self) -> Option<Result<u8, ParseIntError>>;

    /// Parses the json value as `u16`.
    fn parse_u16(&self) -> Option<Result<u16, ParseIntError>>;

    /// Parses the json value as `u32`.
    fn parse_u32(&self) -> Option<Result<u32, ParseIntError>>;

    /// Parses the json value as `u64`.
    fn parse_u64(&self) -> Option<Result<u64, ParseIntError>>;

    /// Parses the json value as `usize`.
    fn parse_usize(&self) -> Option<Result<usize, ParseIntError>>;

    /// Parses the json value as `i32`.
    fn parse_i32(&self) -> Option<Result<i32, ParseIntError>>;

    /// Parses the json value as `i64`.
    fn parse_i64(&self) -> Option<Result<i64, ParseIntError>>;

    /// Parses the json value as `f32`.
    fn parse_f32(&self) -> Option<Result<f32, ParseFloatError>>;

    /// Parses the json value as `f64`.
    fn parse_f64(&self) -> Option<Result<f64, ParseFloatError>>;

    /// Parses the json value as `Cow<'_, str>`.
    /// If the str is empty, it also returns `None`.
    fn parse_string(&self) -> Option<Cow<'_, str>>;

    /// Parses the json value as `Vec<T>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_array<T: FromStr>(&self) -> Option<Vec<T>>;

    /// Parses the json value as `Vec<&str>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_str_array(&self) -> Option<Vec<&str>>;
}

impl JsonValueExt for JsonValue {
    fn parse_bool(&self) -> Option<Result<bool, ParseBoolError>> {
        self.as_bool()
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_u8(&self) -> Option<Result<u8, ParseIntError>> {
        self.as_u64()
            .and_then(|i| u8::try_from(i).ok())
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_u16(&self) -> Option<Result<u16, ParseIntError>> {
        self.as_u64()
            .and_then(|i| u16::try_from(i).ok())
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_u32(&self) -> Option<Result<u32, ParseIntError>> {
        self.as_u64()
            .and_then(|i| u32::try_from(i).ok())
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_u64(&self) -> Option<Result<u64, ParseIntError>> {
        self.as_u64()
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_usize(&self) -> Option<Result<usize, ParseIntError>> {
        self.as_u64()
            .and_then(|i| usize::try_from(i).ok())
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_i32(&self) -> Option<Result<i32, ParseIntError>> {
        self.as_i64()
            .and_then(|i| i32::try_from(i).ok())
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_i64(&self) -> Option<Result<i64, ParseIntError>> {
        self.as_i64()
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_f32(&self) -> Option<Result<f32, ParseFloatError>> {
        self.as_f64()
            .map(|f| Ok(f as f32))
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_f64(&self) -> Option<Result<f64, ParseFloatError>> {
        self.as_f64()
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_string(&self) -> Option<Cow<'_, str>> {
        self.as_str()
            .map(|s| Cow::Borrowed(s.trim()))
            .or_else(|| Some(self.to_string().into()))
            .filter(|s| !s.is_empty())
    }

    fn parse_array<T: FromStr>(&self) -> Option<Vec<T>> {
        let values = match &self {
            JsonValue::String(s) => Some(crate::format::parse_str_array(s)),
            JsonValue::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
            _ => None,
        };
        let vec = values?
            .iter()
            .filter_map(|s| if s.is_empty() { None } else { s.parse().ok() })
            .collect::<Vec<_>>();
        (!vec.is_empty()).then_some(vec)
    }

    fn parse_str_array(&self) -> Option<Vec<&str>> {
        let values = match &self {
            JsonValue::String(s) => Some(crate::format::parse_str_array(s)),
            JsonValue::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
            _ => None,
        };
        let vec = values?
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        (!vec.is_empty()).then_some(vec)
    }
}
