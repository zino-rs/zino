use crate::{datetime::DateTime, response::Response, BoxError, Map, SharedString};
use bytes::Bytes;
use http_body::Full;
use serde_json::Value;
use std::{
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
};
use url::{self, Url};
use uuid::Uuid;

/// A record of validation results.
#[derive(Debug)]
pub struct Validation {
    failed_entries: Vec<(SharedString, BoxError)>,
}

impl Validation {
    /// Creates a new validation record.
    pub fn new() -> Self {
        Self {
            failed_entries: Vec::new(),
        }
    }

    /// Parses a json value as `i64`.
    pub fn parse_i64<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<i64, ParseIntError>> {
        let value = value.into();
        match value.and_then(|v| v.as_i64()) {
            Some(value) => Some(Ok(value)),
            None => value.and_then(|v| v.as_str()).map(|s| s.parse()),
        }
    }

    /// Parses a json value as `u64`.
    pub fn parse_u64<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<u64, ParseIntError>> {
        let value = value.into();
        match value.and_then(|v| v.as_u64()) {
            Some(value) => Some(Ok(value)),
            None => value.and_then(|v| v.as_str()).map(|s| s.parse()),
        }
    }

    /// Parses a json value as `f64`.
    pub fn parse_f64<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<f64, ParseFloatError>> {
        let value = value.into();
        match value.and_then(|v| v.as_f64()) {
            Some(value) => Some(Ok(value)),
            None => value.and_then(|v| v.as_str()).map(|s| s.parse()),
        }
    }

    /// Parses a json value as `bool`.
    pub fn parse_bool<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<bool, ParseBoolError>> {
        let value = value.into();
        match value.and_then(|v| v.as_bool()) {
            Some(value) => Some(Ok(value)),
            None => value.and_then(|v| v.as_str()).map(|s| s.parse()),
        }
    }

    /// Parses a json value as `String`. If the `String` is empty, it also returns `None`.
    pub fn parse_string<'a>(value: impl Into<Option<&'a Value>>) -> Option<String> {
        value
            .into()
            .and_then(|v| {
                v.as_str()
                    .map(|s| s.to_owned())
                    .or_else(|| Some(v.to_string()))
            })
            .filter(|s| !s.is_empty())
    }

    /// Parses a json value as `Vec`. If the vec is empty, it also returns `None`.
    pub fn parse_array<'a, T: FromStr>(value: impl Into<Option<&'a Value>>) -> Option<Vec<T>> {
        value
            .into()
            .and_then(|v| match v {
                Value::String(s) => Some(s.split(',').collect::<Vec<_>>()),
                Value::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
                _ => None,
            })
            .and_then(|values| {
                let vec = values
                    .iter()
                    .filter_map(|s| if s.is_empty() { None } else { s.parse().ok() })
                    .collect::<Vec<_>>();
                (!vec.is_empty()).then_some(vec)
            })
    }

    /// Parses a json value as `Map`. If the map is empty, it also returns `None`.
    pub fn parse_object<'a>(value: impl Into<Option<&'a Value>>) -> Option<&'a Map> {
        value
            .into()
            .and_then(|v| v.as_object())
            .filter(|o| !o.is_empty())
    }

    /// Parses a json value as `Uuid`. If the `Uuid` is `nil`, it also returns `None`.
    pub fn parse_uuid<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<Uuid, uuid::Error>> {
        value
            .into()
            .and_then(|v| v.as_str())
            .map(|s| s.trim_start_matches("urn:uuid:"))
            .filter(|s| !s.chars().all(|c| c == '0' || c == '-'))
            .map(|s| s.parse())
    }

    /// Parses a json value as `DateTime`.
    pub fn parse_datetime<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<DateTime, chrono::format::ParseError>> {
        value.into().and_then(|v| v.as_str()).map(|s| s.parse())
    }

    /// Parses a json value as `Url`.
    pub fn parse_url<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<Url, url::ParseError>> {
        value.into().and_then(|v| v.as_str()).map(|s| s.parse())
    }

    /// Parses a json value as `IpAddr`.
    pub fn parse_ip<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<IpAddr, AddrParseError>> {
        value.into().and_then(|v| v.as_str()).map(|s| s.parse())
    }

    /// Parses a json value as `Ipv4Addr`.
    pub fn parse_ipv4<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<Ipv4Addr, AddrParseError>> {
        value.into().and_then(|v| v.as_str()).map(|s| s.parse())
    }

    /// Parses a json value as `Ipv6Addr`.
    pub fn parse_ipv6<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<Ipv6Addr, AddrParseError>> {
        value.into().and_then(|v| v.as_str()).map(|s| s.parse())
    }

    /// Returns `true` if the validation is success.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.failed_entries.is_empty()
    }

    /// Records a failed entry.
    #[inline]
    pub fn record_fail(&mut self, key: impl Into<SharedString>, err: impl Into<BoxError>) {
        self.failed_entries.push((key.into(), err.into()));
    }

    /// Consumes the validation and returns as a json object.
    #[must_use]
    pub fn into_map(self) -> Map {
        let mut map = Map::new();
        for (key, err) in self.failed_entries {
            map.insert(key.into_owned(), err.to_string().into());
        }
        map
    }
}

impl Default for Validation {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<Validation> for http::Response<Full<Bytes>> {
    #[inline]
    fn from(validation: Validation) -> Self {
        Response::from(validation).into()
    }
}
