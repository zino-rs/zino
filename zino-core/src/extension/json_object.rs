use crate::{
    datetime::{self, DateTime},
    JsonValue, Map, Record, Uuid,
};
use std::{
    borrow::Cow,
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
    time::Duration,
};
use url::{self, Url};

/// Extension trait for [`Map`](crate::Map).
pub trait JsonObjectExt {
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
    /// represents it as `i32` if possible.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Extracts the integer value corresponding to the key.
    fn get_i64(&self, key: &str) -> Option<i64>;

    /// Extracts the float value corresponding to the key and
    /// represents it as `f32` if possible.
    fn get_f32(&self, key: &str) -> Option<f32>;

    /// Extracts the float value corresponding to the key.
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Extracts the array value corresponding to the key.
    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<&str>`.
    fn get_str_array(&self, key: &str) -> Option<Vec<&str>>;

    /// Extracts the object value corresponding to the key.
    fn get_object(&self, key: &str) -> Option<&Map>;

    /// Extracts the value corresponding to the key and parses it as `bool`.
    fn parse_bool(&self, key: &str) -> Option<Result<bool, ParseBoolError>>;

    /// Extracts the value corresponding to the key and parses it as `u8`.
    fn parse_u8(&self, key: &str) -> Option<Result<u8, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `u16`.
    fn parse_u16(&self, key: &str) -> Option<Result<u16, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `u32`.
    fn parse_u32(&self, key: &str) -> Option<Result<u32, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `u64`.
    fn parse_u64(&self, key: &str) -> Option<Result<u64, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `usize`.
    fn parse_usize(&self, key: &str) -> Option<Result<usize, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `i32`.
    fn parse_i32(&self, key: &str) -> Option<Result<i32, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `i64`.
    fn parse_i64(&self, key: &str) -> Option<Result<i64, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `f32`.
    fn parse_f32(&self, key: &str) -> Option<Result<f32, ParseFloatError>>;

    /// Extracts the value corresponding to the key and parses it as `f64`.
    fn parse_f64(&self, key: &str) -> Option<Result<f64, ParseFloatError>>;

    /// Extracts the value corresponding to the key and parses it as `Cow<'_, str>`.
    /// If the str is empty, it also returns `None`.
    fn parse_string(&self, key: &str) -> Option<Cow<'_, str>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<T>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_array<T: FromStr>(&self, key: &str) -> Option<Vec<T>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<&str>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_str_array(&self, key: &str) -> Option<Vec<&str>>;

    /// Extracts the object value corresponding to the key and parses it as `Map`.
    /// If the map is empty, it also returns `None`.
    fn parse_object(&self, key: &str) -> Option<&Map>;

    /// Extracts the string corresponding to the key and parses it as `Uuid`.
    /// If the `Uuid` is `nil`, it also returns `None`.
    fn parse_uuid(&self, key: &str) -> Option<Result<Uuid, uuid::Error>>;

    /// Extracts the string corresponding to the key and parses it as `DateTime`.
    fn parse_datetime(&self, key: &str) -> Option<Result<DateTime, chrono::format::ParseError>>;

    /// Extracts the string corresponding to the key and parses it as `Duration`.
    fn parse_duration(&self, key: &str) -> Option<Result<Duration, datetime::ParseDurationError>>;

    /// Extracts the string corresponding to the key and parses it as `Url`.
    fn parse_url(&self, key: &str) -> Option<Result<Url, url::ParseError>>;

    /// Extracts the string corresponding to the key and parses it as `IpAddr`.
    fn parse_ip(&self, key: &str) -> Option<Result<IpAddr, AddrParseError>>;

    /// Extracts the string corresponding to the key and parses it as `Ipv4Addr`.
    fn parse_ipv4(&self, key: &str) -> Option<Result<Ipv4Addr, AddrParseError>>;

    /// Extracts the string corresponding to the key and parses it as `Ipv6Addr`.
    fn parse_ipv6(&self, key: &str) -> Option<Result<Ipv6Addr, AddrParseError>>;

    /// Looks up a value by a JSON Pointer.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`.
    /// The addressed value is returned and if there is no such value `None` is returned.
    fn lookup(&self, pointer: &str) -> Option<&JsonValue>;

    /// Inserts or updates a  pair into the map.
    /// If the map did have this key present, the value is updated and the old value is returned,
    /// otherwise `None` is returned.
    fn upsert(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Option<JsonValue>;

    /// Consumes `self` and constructs an Avro record value.
    fn into_avro_record(self) -> Record;

    /// Creates a new instance with the entry.
    fn from_entry(key: impl Into<String>, value: impl Into<JsonValue>) -> Self;

    /// Creates a new instance with a single key `entry`.
    fn data_entry(value: impl Into<JsonValue>) -> Self;

    /// Creates a new instance with a single key `entries`.
    fn data_entries(value: Vec<Map>) -> Self;
}

impl JsonObjectExt for Map {
    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    #[inline]
    fn get_u8(&self, key: &str) -> Option<u8> {
        self.get(key)
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok())
    }

    #[inline]
    fn get_u16(&self, key: &str) -> Option<u16> {
        self.get(key)
            .and_then(|v| v.as_u64())
            .and_then(|i| u16::try_from(i).ok())
    }

    #[inline]
    fn get_u32(&self, key: &str) -> Option<u32> {
        self.get(key)
            .and_then(|v| v.as_u64())
            .and_then(|i| u32::try_from(i).ok())
    }

    #[inline]
    fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }

    #[inline]
    fn get_usize(&self, key: &str) -> Option<usize> {
        self.get(key)
            .and_then(|v| v.as_u64())
            .and_then(|i| usize::try_from(i).ok())
    }

    #[inline]
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key)
            .and_then(|v| v.as_i64())
            .and_then(|i| i32::try_from(i).ok())
    }

    #[inline]
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| v.as_f64()).map(|f| f as f32)
    }

    #[inline]
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_f64())
    }

    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>> {
        self.get(key).and_then(|v| v.as_array())
    }

    #[inline]
    fn get_str_array(&self, key: &str) -> Option<Vec<&str>> {
        self.get_array(key)
            .map(|values| values.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
    }

    #[inline]
    fn get_object(&self, key: &str) -> Option<&Map> {
        self.get(key).and_then(|v| v.as_object())
    }

    fn parse_bool(&self, key: &str) -> Option<Result<bool, ParseBoolError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_bool())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_u8(&self, key: &str) -> Option<Result<u8, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_u16(&self, key: &str) -> Option<Result<u16, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| u16::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_u32(&self, key: &str) -> Option<Result<u32, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| u32::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_u64(&self, key: &str) -> Option<Result<u64, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_u64())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_usize(&self, key: &str) -> Option<Result<usize, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_u64())
            .and_then(|i| usize::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_i32(&self, key: &str) -> Option<Result<i32, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_i64())
            .and_then(|i| i32::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_i64(&self, key: &str) -> Option<Result<i64, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_i64())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_f32(&self, key: &str) -> Option<Result<f32, ParseFloatError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_f64())
            .map(|f| Ok(f as f32))
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_f64(&self, key: &str) -> Option<Result<f64, ParseFloatError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_f64())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_string(&self, key: &str) -> Option<Cow<'_, str>> {
        self.get(key)
            .and_then(|v| {
                v.as_str()
                    .map(|s| Cow::Borrowed(s.trim()))
                    .or_else(|| Some(v.to_string().into()))
            })
            .filter(|s| !s.is_empty())
    }

    fn parse_array<T: FromStr>(&self, key: &str) -> Option<Vec<T>> {
        self.get(key)
            .and_then(|v| match v {
                JsonValue::String(s) => Some(crate::format::parse_str_array(s)),
                JsonValue::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
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

    fn parse_str_array(&self, key: &str) -> Option<Vec<&str>> {
        self.get(key)
            .and_then(|v| match v {
                JsonValue::String(s) => Some(crate::format::parse_str_array(s)),
                JsonValue::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
                _ => None,
            })
            .and_then(|values| {
                let vec = values.iter().map(|s| s.trim()).collect::<Vec<_>>();
                (!vec.is_empty()).then_some(vec)
            })
    }

    #[inline]
    fn parse_object(&self, key: &str) -> Option<&Map> {
        self.get_object(key).filter(|o| !o.is_empty())
    }

    fn parse_uuid(&self, key: &str) -> Option<Result<Uuid, uuid::Error>> {
        self.get_str(key)
            .map(|s| s.trim_start_matches("urn:uuid:"))
            .filter(|s| !s.chars().all(|c| c == '0' || c == '-'))
            .map(|s| s.parse())
    }

    #[inline]
    fn parse_datetime(&self, key: &str) -> Option<Result<DateTime, chrono::format::ParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_duration(&self, key: &str) -> Option<Result<Duration, datetime::ParseDurationError>> {
        self.get_str(key).map(datetime::parse_duration)
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

    fn lookup(&self, pointer: &str) -> Option<&JsonValue> {
        let Some(path) = pointer.strip_prefix('/') else {
            return None;
        };
        if let Some(position) = path.find('/') {
            let (key, pointer) = path.split_at(position);
            self.get(key)?.pointer(pointer)
        } else {
            self.get(path)
        }
    }

    #[inline]
    fn upsert(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Option<JsonValue> {
        self.insert(key.into(), value.into())
    }

    fn into_avro_record(self) -> Record {
        let mut record = Record::with_capacity(self.len());
        for (field, value) in self.into_iter() {
            record.push((field, value.into()));
        }
        record
    }

    #[inline]
    fn from_entry(key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        let mut map = Map::with_capacity(1);
        map.insert(key.into(), value.into());
        map
    }

    #[inline]
    fn data_entry(value: impl Into<JsonValue>) -> Self {
        let mut map = Map::with_capacity(1);
        map.insert("entry".to_owned(), value.into());
        map
    }

    #[inline]
    fn data_entries(value: Vec<Map>) -> Self {
        let mut map = Map::with_capacity(1);
        map.insert("entries".to_owned(), value.into());
        map
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        extension::{JsonObjectExt, JsonValueExt},
        Map,
    };

    #[test]
    fn it_lookups_json_value() {
        let mut map = Map::new();
        map.upsert("entries", vec![Map::from_entry("name", "alice")]);
        map.upsert("total", 1);

        assert_eq!(map.lookup("total"), None);
        assert_eq!(map.lookup("/total").and_then(|v| v.as_usize()), Some(1));
        assert_eq!(
            map.lookup("/entries/0/name").and_then(|v| v.as_str()),
            Some("alice")
        );
    }
}
