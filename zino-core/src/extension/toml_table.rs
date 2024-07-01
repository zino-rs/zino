use crate::{datetime, extension::TomlValueExt, Map, Uuid};
use std::{
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};
use toml::value::{Array, Table};
use url::Url;

/// Extension trait for [`Table`](toml::Table).
pub trait TomlTableExt {
    /// Extracts the boolean value corresponding to the key.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `u8` if possible.
    fn get_u8(&self, key: &str) -> Option<u8>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `u16` if possible.
    fn get_u16(&self, key: &str) -> Option<u16>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `u32` if possible.
    fn get_u32(&self, key: &str) -> Option<u32>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `u64` if possible.
    fn get_u64(&self, key: &str) -> Option<u64>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `usize` if possible.
    fn get_usize(&self, key: &str) -> Option<usize>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `i8` if possible.
    fn get_i8(&self, key: &str) -> Option<i8>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `i16` if possible.
    fn get_i16(&self, key: &str) -> Option<i16>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `i32` if possible.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Extracts the integer value corresponding to the key.
    fn get_i64(&self, key: &str) -> Option<i64>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `isize` if possible.
    fn get_isize(&self, key: &str) -> Option<isize>;

    /// Extracts the float value corresponding to the key and
    /// represents it as `f32` if possible.
    fn get_f32(&self, key: &str) -> Option<f32>;

    /// Extracts the float value corresponding to the key.
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Extracts the array corresponding to the key.
    fn get_array(&self, key: &str) -> Option<&Array>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<&str>`.
    fn get_str_array(&self, key: &str) -> Option<Vec<&str>>;

    /// Extracts the table corresponding to the key.
    fn get_table(&self, key: &str) -> Option<&Table>;

    /// Extracts the first table in an array corresponding to the key.
    fn get_first_table(&self, key: &str) -> Option<&Table>;

    /// Extracts the last table in an array corresponding to the key.
    fn get_last_table(&self, key: &str) -> Option<&Table>;

    /// Extracts the string corresponding to the key
    /// and parses it as `Duration`.
    fn get_duration(&self, key: &str) -> Option<Duration>;

    /// Extracts the string corresponding to the key and parses it as `Uuid`.
    /// If the `Uuid` is `nil`, it also returns `None`.
    fn parse_uuid(&self, key: &str) -> Option<Result<Uuid, uuid::Error>>;

    /// Extracts the string corresponding to the key and parses it as `Url`.
    fn parse_url(&self, key: &str) -> Option<Result<Url, url::ParseError>>;

    /// Extracts the string corresponding to the key and parses it as `IpAddr`.
    fn parse_ip(&self, key: &str) -> Option<Result<IpAddr, AddrParseError>>;

    /// Extracts the string corresponding to the key and parses it as `Ipv4Addr`.
    fn parse_ipv4(&self, key: &str) -> Option<Result<Ipv4Addr, AddrParseError>>;

    /// Extracts the string corresponding to the key and parses it as `Ipv6Addr`.
    fn parse_ipv6(&self, key: &str) -> Option<Result<Ipv6Addr, AddrParseError>>;

    /// Converts `self` to a JSON object.
    fn to_map(&self) -> Map;
}

impl TomlTableExt for Table {
    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    #[inline]
    fn get_u8(&self, key: &str) -> Option<u8> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| u8::try_from(i).ok())
    }

    #[inline]
    fn get_u16(&self, key: &str) -> Option<u16> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| u16::try_from(i).ok())
    }

    #[inline]
    fn get_u32(&self, key: &str) -> Option<u32> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| u32::try_from(i).ok())
    }

    #[inline]
    fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| u64::try_from(i).ok())
    }

    #[inline]
    fn get_usize(&self, key: &str) -> Option<usize> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| usize::try_from(i).ok())
    }

    #[inline]
    fn get_i8(&self, key: &str) -> Option<i8> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| i8::try_from(i).ok())
    }

    #[inline]
    fn get_i16(&self, key: &str) -> Option<i16> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| i16::try_from(i).ok())
    }

    #[inline]
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| i32::try_from(i).ok())
    }

    #[inline]
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_integer())
    }

    #[inline]
    fn get_isize(&self, key: &str) -> Option<isize> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| isize::try_from(i).ok())
    }

    #[inline]
    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| v.as_float()).map(|f| f as f32)
    }

    #[inline]
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_float())
    }

    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn get_array(&self, key: &str) -> Option<&Array> {
        self.get(key).and_then(|v| v.as_array())
    }

    #[inline]
    fn get_str_array(&self, key: &str) -> Option<Vec<&str>> {
        self.get_array(key)
            .map(|values| values.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
    }

    #[inline]
    fn get_table(&self, key: &str) -> Option<&Table> {
        self.get(key).and_then(|v| v.as_table())
    }

    #[inline]
    fn get_first_table(&self, key: &str) -> Option<&Table> {
        self.get_array(key)?.first()?.as_table()
    }

    #[inline]
    fn get_last_table(&self, key: &str) -> Option<&Table> {
        self.get_array(key)?.last()?.as_table()
    }

    fn get_duration(&self, key: &str) -> Option<Duration> {
        self.get_str(key)
            .and_then(|s| datetime::parse_duration(s).ok())
    }

    fn parse_uuid(&self, key: &str) -> Option<Result<Uuid, uuid::Error>> {
        self.get_str(key)
            .map(|s| s.trim_start_matches("urn:uuid:"))
            .filter(|s| !s.chars().all(|c| c == '0' || c == '-'))
            .map(|s| s.parse())
    }

    #[inline]
    fn parse_url(&self, key: &str) -> Option<Result<Url, url::ParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_ip(&self, key: &str) -> Option<Result<IpAddr, AddrParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_ipv4(&self, key: &str) -> Option<Result<Ipv4Addr, AddrParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_ipv6(&self, key: &str) -> Option<Result<Ipv6Addr, AddrParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    fn to_map(&self) -> Map {
        let mut map = Map::new();
        for (key, value) in self.iter() {
            map.insert(key.to_owned(), value.to_json_value());
        }
        map
    }
}
