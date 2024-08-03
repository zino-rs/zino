use super::JsonValueExt;
use crate::{
    datetime::{self, Date, DateTime, Time},
    helper,
    model::Model,
    validation::Validation,
    JsonValue, Map, Record, Uuid,
};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::{
    borrow::Cow,
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
    time::Duration,
};
use url::Url;

/// Extension trait for [`Map`](crate::Map).
pub trait JsonObjectExt {
    /// Extracts the boolean value corresponding to the key.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `u8` if possible.
    fn get_u8(&self, key: &str) -> Option<u8>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `u16` if possible.
    fn get_u16(&self, key: &str) -> Option<u16>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `u32` if possible.
    fn get_u32(&self, key: &str) -> Option<u32>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `u64` if possible.
    fn get_u64(&self, key: &str) -> Option<u64>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `usize` if possible.
    fn get_usize(&self, key: &str) -> Option<usize>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `i8` if possible.
    fn get_i8(&self, key: &str) -> Option<i8>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `i16` if possible.
    fn get_i16(&self, key: &str) -> Option<i16>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `i32` if possible.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Extracts the integer value corresponding to the key.
    fn get_i64(&self, key: &str) -> Option<i64>;

    /// Extracts the integer value corresponding to the key
    /// and represents it as `isize` if possible.
    fn get_isize(&self, key: &str) -> Option<isize>;

    /// Extracts the float value corresponding to the key
    /// and represents it as `f32` if possible.
    fn get_f32(&self, key: &str) -> Option<f32>;

    /// Extracts the float value corresponding to the key.
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Extracts the string corresponding to the key
    /// and represents it as `Uuid` if possible.
    fn get_uuid(&self, key: &str) -> Option<Uuid>;

    /// Extracts the string corresponding to the key
    /// and represents it as `Date` if possible.
    fn get_date(&self, key: &str) -> Option<Date>;

    /// Extracts the string corresponding to the key
    /// and represents it as `Time` if possible.
    fn get_time(&self, key: &str) -> Option<Time>;

    /// Extracts the string corresponding to the key
    /// and represents it as `DateTime` if possible.
    fn get_date_time(&self, key: &str) -> Option<DateTime>;

    /// Extracts the string corresponding to the key
    /// and represents it as `NaiveDateTime` if possible.
    fn get_naive_date_time(&self, key: &str) -> Option<NaiveDateTime>;

    /// Extracts the string corresponding to the key
    /// and represents it as `Duration` if possible.
    fn get_duration(&self, key: &str) -> Option<Duration>;

    /// Extracts the array value corresponding to the key.
    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<u64>`.
    fn get_u64_array(&self, key: &str) -> Option<Vec<u64>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<i64>`.
    fn get_i64_array(&self, key: &str) -> Option<Vec<i64>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<f32>`.
    fn get_f32_array(&self, key: &str) -> Option<Vec<f32>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<f64>`.
    fn get_f64_array(&self, key: &str) -> Option<Vec<f64>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<&str>`.
    fn get_str_array(&self, key: &str) -> Option<Vec<&str>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<&Map>`.
    fn get_map_array(&self, key: &str) -> Option<Vec<&Map>>;

    /// Extracts the object value corresponding to the key.
    fn get_object(&self, key: &str) -> Option<&Map>;

    /// Extracts the populated data corresponding to the key.
    fn get_populated(&self, key: &str) -> Option<&Map>;

    /// Extracts the translated string value corresponding to the key.
    fn get_translated(&self, key: &str) -> Option<&str>;

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

    /// Extracts the value corresponding to the key and parses it as `i8`.
    fn parse_i8(&self, key: &str) -> Option<Result<i8, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `i16`.
    fn parse_i16(&self, key: &str) -> Option<Result<i16, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `i32`.
    fn parse_i32(&self, key: &str) -> Option<Result<i32, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `i64`.
    fn parse_i64(&self, key: &str) -> Option<Result<i64, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `isize`.
    fn parse_isize(&self, key: &str) -> Option<Result<isize, ParseIntError>>;

    /// Extracts the value corresponding to the key and parses it as `f32`.
    fn parse_f32(&self, key: &str) -> Option<Result<f32, ParseFloatError>>;

    /// Extracts the value corresponding to the key and parses it as `f64`.
    fn parse_f64(&self, key: &str) -> Option<Result<f64, ParseFloatError>>;

    /// Extracts the value corresponding to the key and parses it as `Cow<'_, str>`.
    /// If the str is empty, it also returns `None`.
    fn parse_string(&self, key: &str) -> Option<Cow<'_, str>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<T>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_array<T: FromStr>(&self, key: &str) -> Option<Result<Vec<T>, <T as FromStr>::Err>>;

    /// Extracts the array value corresponding to the key and parses it as `Vec<&str>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_str_array(&self, key: &str) -> Option<Vec<&str>>;

    /// Extracts the enum values corresponding to the key
    /// and parses it as `Vec<i64>` or `Vec<String>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_enum_values(&self, key: &str) -> Option<Vec<JsonValue>>;

    /// Extracts the object value corresponding to the key and parses it as `Map`.
    /// If the map is empty, it also returns `None`.
    fn parse_object(&self, key: &str) -> Option<&Map>;

    /// Extracts the string corresponding to the key and parses it as `Uuid`.
    /// If the `Uuid` is `nil`, it also returns `None`.
    fn parse_uuid(&self, key: &str) -> Option<Result<Uuid, uuid::Error>>;

    /// Extracts the string corresponding to the key and parses it as `Decimal`.
    fn parse_decimal(&self, key: &str) -> Option<Result<Decimal, rust_decimal::Error>>;

    /// Extracts the string corresponding to the key and parses it as `Date`.
    fn parse_date(&self, key: &str) -> Option<Result<Date, chrono::format::ParseError>>;

    /// Extracts the string corresponding to the key and parses it as `Time`.
    fn parse_time(&self, key: &str) -> Option<Result<Time, chrono::format::ParseError>>;

    /// Extracts the string corresponding to the key and parses it as `DateTime`.
    fn parse_date_time(&self, key: &str) -> Option<Result<DateTime, chrono::format::ParseError>>;

    /// Extracts the string corresponding to the key and parses it as `NaiveDateTime`.
    fn parse_naive_date_time(
        &self,
        key: &str,
    ) -> Option<Result<NaiveDateTime, chrono::format::ParseError>>;

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

    /// Extracts the value corresponding to the key and parses it as a model `M`.
    fn parse_model<M: Model>(&self, key: &str) -> Option<Result<M, Validation>>;

    /// Looks up a value by a JSON Pointer.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`.
    /// The addressed value is returned and if there is no such value `None` is returned.
    fn pointer(&self, pointer: &str) -> Option<&JsonValue>;

    /// Inserts or updates a  pair into the map.
    /// If the map did have this key present, the value is updated and the old value is returned,
    /// otherwise `None` is returned.
    fn upsert(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Option<JsonValue>;

    /// Copies values from the populated data corresponding to the key into `self`.
    fn clone_from_populated(&mut self, key: &str, fields: &[&str]);

    /// Extracts values from the populated data corresponding to the key and moves them to `self`.
    fn extract_from_populated(&mut self, key: &str, fields: &[&str]);

    /// Translates the map with the OpenAPI data.
    #[cfg(feature = "openapi")]
    fn translate_with_openapi(&mut self, name: &str);

    /// Attempts to read the map as an instance of the model `M`.
    fn read_as_model<M: Model>(&self) -> Result<M, Validation>;

    /// Serializes the map into a string.
    fn to_string(&self) -> String;

    /// Serializes the map into a query string.
    fn to_query_string(&self) -> String;

    /// Consumes `self` and constructs an Avro record value.
    fn into_avro_record(self) -> Record;

    /// Creates a new instance with the entry.
    fn from_entry(key: impl Into<String>, value: impl Into<JsonValue>) -> Self;

    /// Creates a new instance from the entries.
    /// If the JSON value is not an object, an empty map will be returned.
    fn from_entries(entries: JsonValue) -> Self;

    /// Creates a new instance with a single key `entry`.
    fn data_entry(value: Map) -> Self;

    /// Creates a new instance with the `entries`.
    fn data_entries(values: Vec<Map>) -> Self;

    /// Creates a new instance with a single key `item`.
    fn data_item(value: impl Into<JsonValue>) -> Self;

    /// Creates a new instance with the `items`.
    fn data_items<T: Into<JsonValue>>(values: Vec<T>) -> Self;
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
    fn get_i8(&self, key: &str) -> Option<i8> {
        self.get(key)
            .and_then(|v| v.as_i64())
            .and_then(|i| i8::try_from(i).ok())
    }

    #[inline]
    fn get_i16(&self, key: &str) -> Option<i16> {
        self.get(key)
            .and_then(|v| v.as_i64())
            .and_then(|i| i16::try_from(i).ok())
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
    fn get_isize(&self, key: &str) -> Option<isize> {
        self.get(key)
            .and_then(|v| v.as_i64())
            .and_then(|i| isize::try_from(i).ok())
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
    fn get_uuid(&self, key: &str) -> Option<Uuid> {
        self.get_str(key).and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_date(&self, key: &str) -> Option<Date> {
        self.get_str(key).and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_time(&self, key: &str) -> Option<Time> {
        self.get_str(key).and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_date_time(&self, key: &str) -> Option<DateTime> {
        self.get_str(key).and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_naive_date_time(&self, key: &str) -> Option<NaiveDateTime> {
        self.get_str(key).and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_duration(&self, key: &str) -> Option<Duration> {
        self.get_str(key)
            .and_then(|s| datetime::parse_duration(s).ok())
    }

    #[inline]
    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>> {
        self.get(key).and_then(|v| v.as_array())
    }

    #[inline]
    fn get_u64_array(&self, key: &str) -> Option<Vec<u64>> {
        self.get_array(key)
            .map(|values| values.iter().filter_map(|v| v.as_u64()).collect())
    }

    #[inline]
    fn get_i64_array(&self, key: &str) -> Option<Vec<i64>> {
        self.get_array(key)
            .map(|values| values.iter().filter_map(|v| v.as_i64()).collect())
    }

    #[inline]
    fn get_f32_array(&self, key: &str) -> Option<Vec<f32>> {
        self.get_array(key).map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        })
    }

    #[inline]
    fn get_f64_array(&self, key: &str) -> Option<Vec<f64>> {
        self.get_array(key)
            .map(|values| values.iter().filter_map(|v| v.as_f64()).collect())
    }

    #[inline]
    fn get_str_array(&self, key: &str) -> Option<Vec<&str>> {
        self.get_array(key)
            .map(|values| values.iter().filter_map(|v| v.as_str()).collect())
    }

    #[inline]
    fn get_map_array(&self, key: &str) -> Option<Vec<&Map>> {
        self.get_array(key).map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_object())
                .collect::<Vec<_>>()
        })
    }

    #[inline]
    fn get_object(&self, key: &str) -> Option<&Map> {
        self.get(key).and_then(|v| v.as_object())
    }

    #[inline]
    fn get_populated(&self, key: &str) -> Option<&Map> {
        let populated_field = [key, "_populated"].concat();
        self.get_object(&populated_field)
    }

    #[inline]
    fn get_translated(&self, key: &str) -> Option<&str> {
        let translated_field = [key, "_translated"].concat();
        self.get_str(&translated_field)
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

    fn parse_i8(&self, key: &str) -> Option<Result<i8, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_i64())
            .and_then(|i| i8::try_from(i).ok())
            .map(Ok)
            .or_else(|| value.and_then(|v| v.as_str()).map(|s| s.parse()))
    }

    fn parse_i16(&self, key: &str) -> Option<Result<i16, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_i64())
            .and_then(|i| i16::try_from(i).ok())
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

    fn parse_isize(&self, key: &str) -> Option<Result<isize, ParseIntError>> {
        let value = self.get(key);
        value
            .and_then(|v| v.as_i64())
            .and_then(|i| isize::try_from(i).ok())
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

    fn parse_array<T: FromStr>(&self, key: &str) -> Option<Result<Vec<T>, <T as FromStr>::Err>> {
        let values = match self.get(key)? {
            JsonValue::String(s) => helper::parse_str_array(s, ',')
                .into_iter()
                .filter_map(|s| (!s.is_empty()).then_some(Cow::Borrowed(s)))
                .collect::<Vec<_>>(),
            JsonValue::Array(vec) => vec
                .iter()
                .filter(|v| !v.is_null())
                .filter_map(|v| v.parse_string())
                .collect::<Vec<_>>(),
            _ => return None,
        };
        let mut vec = Vec::with_capacity(values.len());
        for value in values {
            match value.parse() {
                Ok(v) => vec.push(v),
                Err(err) => return Some(Err(err)),
            }
        }
        (!vec.is_empty()).then_some(Ok(vec))
    }

    fn parse_str_array(&self, key: &str) -> Option<Vec<&str>> {
        self.get(key)
            .and_then(|v| match v {
                JsonValue::String(s) => Some(helper::parse_str_array(s, ',')),
                JsonValue::Array(v) => Some(v.iter().filter_map(|v| v.as_str()).collect()),
                _ => None,
            })
            .and_then(|values| {
                let vec = values.iter().map(|s| s.trim()).collect::<Vec<_>>();
                (!vec.is_empty()).then_some(vec)
            })
    }

    fn parse_enum_values(&self, key: &str) -> Option<Vec<JsonValue>> {
        self.get(key)
            .and_then(|v| match v {
                JsonValue::String(s) => {
                    let values = helper::parse_str_array(s, '|');
                    let vec = values
                        .iter()
                        .map(|s| {
                            let s = s.trim();
                            if let Ok(integer) = s.parse::<i64>() {
                                JsonValue::Number(integer.into())
                            } else {
                                JsonValue::String(s.to_owned())
                            }
                        })
                        .collect::<Vec<_>>();
                    Some(vec)
                }
                JsonValue::Array(vec) => Some(vec.clone()),
                _ => None,
            })
            .filter(|vec| !vec.is_empty())
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
    fn parse_decimal(&self, key: &str) -> Option<Result<Decimal, rust_decimal::Error>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_date(&self, key: &str) -> Option<Result<Date, chrono::format::ParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_time(&self, key: &str) -> Option<Result<Time, chrono::format::ParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_date_time(&self, key: &str) -> Option<Result<DateTime, chrono::format::ParseError>> {
        self.get_str(key).map(|s| s.parse())
    }

    #[inline]
    fn parse_naive_date_time(
        &self,
        key: &str,
    ) -> Option<Result<NaiveDateTime, chrono::format::ParseError>> {
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

    fn parse_model<M: Model>(&self, key: &str) -> Option<Result<M, Validation>> {
        self.get_object(key).map(|data| {
            let mut model = M::new();
            let validation = model.read_map(data);
            if validation.is_success() {
                Ok(model)
            } else {
                Err(validation)
            }
        })
    }

    fn pointer(&self, pointer: &str) -> Option<&JsonValue> {
        let path = pointer.strip_prefix('/')?;
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

    fn clone_from_populated(&mut self, key: &str, fields: &[&str]) {
        let mut object = Map::new();
        if let Some(map) = self.get_populated(key) {
            for &field in fields {
                object.upsert(field, map.get(field).cloned());
            }
        }
        self.append(&mut object);
    }

    fn extract_from_populated(&mut self, key: &str, fields: &[&str]) {
        let mut object = Map::new();
        let populated_field = [key, "_populated"].concat();
        if let Some(&mut ref mut map) = self
            .get_mut(&populated_field)
            .and_then(|v| v.as_object_mut())
        {
            for &field in fields {
                object.upsert(field, map.remove(field));
            }
        }
        self.append(&mut object);
    }

    #[cfg(feature = "openapi")]
    #[inline]
    fn translate_with_openapi(&mut self, name: &str) {
        crate::openapi::translate_model_entry(self, name);
    }

    fn read_as_model<M: Model>(&self) -> Result<M, Validation> {
        let mut model = M::new();
        let validation = model.read_map(self);
        if validation.is_success() {
            Ok(model)
        } else {
            Err(validation)
        }
    }

    #[inline]
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    #[inline]
    fn to_query_string(&self) -> String {
        serde_qs::to_string(&self).unwrap_or_default()
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
        let mut map = Map::new();
        map.insert(key.into(), value.into());
        map
    }

    #[inline]
    fn from_entries(entries: JsonValue) -> Self {
        if let JsonValue::Object(map) = entries {
            map
        } else {
            Map::new()
        }
    }

    #[inline]
    fn data_entry(value: Map) -> Self {
        let mut map = Map::new();
        map.insert("entry".to_owned(), value.into());
        map
    }

    #[inline]
    fn data_entries(values: Vec<Map>) -> Self {
        let mut map = Map::new();
        map.insert("num_entries".to_owned(), values.len().into());
        map.insert("entries".to_owned(), values.into());
        map
    }

    #[inline]
    fn data_item(value: impl Into<JsonValue>) -> Self {
        let mut map = Map::new();
        map.insert("item".to_owned(), value.into());
        map
    }

    #[inline]
    fn data_items<T: Into<JsonValue>>(values: Vec<T>) -> Self {
        let mut map = Map::new();
        map.insert("num_items".to_owned(), values.len().into());
        map.insert("items".to_owned(), values.into());
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
    fn it_parses_str_array() {
        let mut map = Map::new();
        map.upsert("roles", vec!["admin", "", "worker"]);

        assert_eq!(
            map.get_str_array("roles"),
            Some(vec!["admin", "", "worker"])
        );
        assert_eq!(
            map.parse_str_array("roles"),
            Some(vec!["admin", "", "worker"])
        );
        assert_eq!(
            map.parse_array::<String>("roles"),
            Some(Ok(vec!["admin".to_owned(), "worker".to_owned()]))
        );
    }

    #[test]
    fn it_lookups_json_value() {
        let mut map = Map::new();
        map.upsert("entries", vec![Map::from_entry("name", "alice")]);
        map.upsert("total", 1);

        assert_eq!(map.pointer("total"), None);
        assert_eq!(map.pointer("/total").and_then(|v| v.as_usize()), Some(1));
        assert_eq!(
            map.pointer("/entries/0/name").and_then(|v| v.as_str()),
            Some("alice")
        );
    }
}
