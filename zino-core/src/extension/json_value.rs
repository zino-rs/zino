use serde_json::Value;
use std::{
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
};

/// Extension trait for [`serde_json::Value`](serde_json::Value).
pub trait JsonValueExt {
    /// Parses the json value as `u32`.
    fn parse_u32(&self) -> Option<Result<u32, ParseIntError>>;

    /// Parses the json value as `u64`.
    fn parse_u64(&self) -> Option<Result<u64, ParseIntError>>;

    /// Parses the json value as `f64`.
    fn parse_f64(&self) -> Option<Result<f64, ParseFloatError>>;

    /// Parses the json value as `bool`.
    fn parse_bool(&self) -> Option<Result<bool, ParseBoolError>>;

    /// Parses the json value as `Vec<T>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_array<T: FromStr>(&self) -> Option<Vec<T>>;

    /// Parses the json value as `Vec<&str>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_str_array(&self) -> Option<Vec<&str>>;
}

impl JsonValueExt for Value {
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

    fn parse_f64(&self) -> Option<Result<f64, ParseFloatError>> {
        self.as_f64()
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_bool(&self) -> Option<Result<bool, ParseBoolError>> {
        self.as_bool()
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_array<T: FromStr>(&self) -> Option<Vec<T>> {
        let values = match &self {
            Value::String(s) => Some(crate::format::parse_str_array(s)),
            Value::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
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
            Value::String(s) => Some(crate::format::parse_str_array(s)),
            Value::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
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
