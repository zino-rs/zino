use crate::{
    datetime::{self, DateTime},
    error::Error,
    extension::JsonObjectExt,
    format::str_array,
    Map, SharedString,
};
use serde_json::Value;
use std::{
    borrow::Cow,
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
    time::Duration,
};
use url::{self, Url};
use uuid::Uuid;

/// A record of validation results.
#[derive(Debug, Default)]
pub struct Validation {
    failed_entries: Vec<(SharedString, Error)>,
}

impl Validation {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            failed_entries: Vec::new(),
        }
    }

    /// Creates a new instance with the entry.
    #[inline]
    pub fn from_entry(key: impl Into<SharedString>, err: impl Into<Error>) -> Self {
        let failed_entries = vec![(key.into(), err.into())];
        Self { failed_entries }
    }

    /// Records an entry with the supplied message.
    #[inline]
    pub fn record(&mut self, key: impl Into<SharedString>, message: impl Into<SharedString>) {
        self.failed_entries.push((key.into(), Error::new(message)));
    }

    /// Records an entry for the error.
    #[inline]
    pub fn record_fail(&mut self, key: impl Into<SharedString>, err: impl Into<Error>) {
        self.failed_entries.push((key.into(), err.into()));
    }

    /// Returns true if the validation contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.failed_entries.iter().any(|(field, _)| field == key)
    }

    /// Returns `true` if the validation is success.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.failed_entries.is_empty()
    }

    /// Consumes the validation and returns as a json object.
    #[must_use]
    pub fn into_map(self) -> Map {
        let failed_entries = self.failed_entries;
        let mut map = Map::with_capacity(failed_entries.len());
        for (key, err) in failed_entries {
            map.upsert(key, err.to_string());
        }
        map
    }

    /// Parses a json value as `i64`.
    pub fn parse_i64<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<i64, ParseIntError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_i64())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `u8`.
    pub fn parse_u8<'a>(value: impl Into<Option<&'a Value>>) -> Option<Result<u8, ParseIntError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `u16`.
    pub fn parse_u16<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<u16, ParseIntError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| u16::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `u32`.
    pub fn parse_u32<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<u32, ParseIntError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| u32::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `u64`.
    pub fn parse_u64<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<u64, ParseIntError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_u64())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `usize`.
    pub fn parse_usize<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<usize, ParseIntError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| usize::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `f32`.
    pub fn parse_f32<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<f32, ParseFloatError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_f64())
            .map(|f| Ok(f as f32))
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `f64`.
    pub fn parse_f64<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<f64, ParseFloatError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_f64())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `bool`.
    pub fn parse_bool<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<bool, ParseBoolError>> {
        let value = value.into();
        value
            .and_then(|v| v.as_bool())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    /// Parses a json value as `Cow<'_, str>`. If the str is empty, it also returns `None`.
    pub fn parse_string<'a>(value: impl Into<Option<&'a Value>>) -> Option<Cow<'a, str>> {
        value
            .into()
            .and_then(|v| {
                v.as_str()
                    .map(|s| Cow::Borrowed(s.trim()))
                    .or_else(|| Some(v.to_string().into()))
            })
            .filter(|s| !s.is_empty())
    }

    /// Parses a json value as `Vec<T>`. If the vec is empty, it also returns `None`.
    pub fn parse_array<'a, T: FromStr>(value: impl Into<Option<&'a Value>>) -> Option<Vec<T>> {
        value
            .into()
            .and_then(|v| match v {
                Value::String(s) => Some(str_array::parse_str_array(s)),
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

    /// Parses a json value as `Vec<&str>`. If the vec is empty, it also returns `None`.
    pub fn parse_str_array<'a>(value: impl Into<Option<&'a Value>>) -> Option<Vec<&'a str>> {
        value
            .into()
            .and_then(|v| match v {
                Value::String(s) => Some(str_array::parse_str_array(s)),
                Value::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
                _ => None,
            })
            .and_then(|values| {
                let vec = values
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
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

    /// Parses a json value as `Duration`.
    pub fn parse_duration<'a>(
        value: impl Into<Option<&'a Value>>,
    ) -> Option<Result<Duration, datetime::ParseDurationError>> {
        value
            .into()
            .and_then(|v| v.as_str())
            .map(datetime::parse_duration)
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
}
