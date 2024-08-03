use crate::{
    datetime::{self, Date, DateTime, Time},
    extension::JsonObjectExt,
    helper, Decimal, JsonValue, Map, Uuid,
};
use chrono::NaiveDateTime;
use csv::{ByteRecord, Writer};
use serde::de::DeserializeOwned;
use std::{
    borrow::Cow,
    io::{self, ErrorKind},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
    time::Duration,
};

/// Extension trait for [`serde_json::Value`].
pub trait JsonValueExt {
    /// Returns `true` if the JSON value can be ignored.
    fn is_ignorable(&self) -> bool;

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

    /// If the `Value` is an integer, represent it as `i8` if possible.
    /// Returns `None` otherwise.
    fn as_i8(&self) -> Option<i8>;

    /// If the `Value` is an integer, represent it as `i16` if possible.
    /// Returns `None` otherwise.
    fn as_i16(&self) -> Option<i16>;

    /// If the `Value` is an integer, represent it as `i32` if possible.
    /// Returns `None` otherwise.
    fn as_i32(&self) -> Option<i32>;

    /// If the `Value` is an integer, represent it as `isize` if possible.
    /// Returns `None` otherwise.
    fn as_isize(&self) -> Option<isize>;

    /// If the `Value` is a float, represent it as `f32` if possible.
    /// Returns `None` otherwise.
    fn as_f32(&self) -> Option<f32>;

    /// If the `Value` is an array of strings, returns the associated vector.
    /// Returns `None` otherwise.
    fn as_str_array(&self) -> Option<Vec<&str>>;

    /// If the `Value` is an array of maps, returns the associated vector.
    /// Returns `None` otherwise.
    fn as_map_array(&self) -> Option<Vec<&Map>>;

    /// If the `Value` is a String, represent it as `Uuid` if possible.
    /// Returns `None` otherwise.
    fn as_uuid(&self) -> Option<Uuid>;

    /// If the `Value` is a String, represent it as `Date` if possible.
    /// Returns `None` otherwise.
    fn as_date(&self) -> Option<Date>;

    /// If the `Value` is a String, represent it as `Time` if possible.
    /// Returns `None` otherwise.
    fn as_time(&self) -> Option<Time>;

    /// If the `Value` is a String, represent it as `DateTime` if possible.
    /// Returns `None` otherwise.
    fn as_date_time(&self) -> Option<DateTime>;

    /// If the `Value` is a String, represent it as `NaiveDateTime` if possible.
    /// Returns `None` otherwise.
    fn as_naive_date_time(&self) -> Option<NaiveDateTime>;

    /// If the `Value` is a String, represent it as `Duration` if possible.
    /// Returns `None` otherwise.
    fn as_duration(&self) -> Option<Duration>;

    /// Parses the JSON value as `bool`.
    fn parse_bool(&self) -> Option<Result<bool, ParseBoolError>>;

    /// Parses the JSON value as `u8`.
    fn parse_u8(&self) -> Option<Result<u8, ParseIntError>>;

    /// Parses the JSON value as `u16`.
    fn parse_u16(&self) -> Option<Result<u16, ParseIntError>>;

    /// Parses the JSON value as `u32`.
    fn parse_u32(&self) -> Option<Result<u32, ParseIntError>>;

    /// Parses the JSON value as `u64`.
    fn parse_u64(&self) -> Option<Result<u64, ParseIntError>>;

    /// Parses the JSON value as `usize`.
    fn parse_usize(&self) -> Option<Result<usize, ParseIntError>>;

    /// Parses the JSON value as `i8`.
    fn parse_i8(&self) -> Option<Result<i8, ParseIntError>>;

    /// Parses the JSON value as `i16`.
    fn parse_i16(&self) -> Option<Result<i16, ParseIntError>>;

    /// Parses the JSON value as `i32`.
    fn parse_i32(&self) -> Option<Result<i32, ParseIntError>>;

    /// Parses the JSON value as `i64`.
    fn parse_i64(&self) -> Option<Result<i64, ParseIntError>>;

    /// Parses the JSON value as `isize`.
    fn parse_isize(&self) -> Option<Result<isize, ParseIntError>>;

    /// Parses the JSON value as `f32`.
    fn parse_f32(&self) -> Option<Result<f32, ParseFloatError>>;

    /// Parses the JSON value as `f64`.
    fn parse_f64(&self) -> Option<Result<f64, ParseFloatError>>;

    /// Parses the JSON value as `Cow<'_, str>`.
    /// If the str is empty, it also returns `None`.
    fn parse_string(&self) -> Option<Cow<'_, str>>;

    /// Parses the JSON value as `Vec<T>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_array<T: FromStr>(&self) -> Option<Result<Vec<T>, <T as FromStr>::Err>>;

    /// Parses the JSON value as `Vec<&str>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_str_array(&self) -> Option<Vec<&str>>;

    /// Parses the JSON value as `Uuid`.
    /// If the `Uuid` is `nil`, it also returns `None`.
    fn parse_uuid(&self) -> Option<Result<Uuid, uuid::Error>>;

    /// Parses the JSON value as `Decimal`.
    fn parse_decimal(&self) -> Option<Result<Decimal, rust_decimal::Error>>;

    /// Parses the JSON value as `Date`.
    fn parse_date(&self) -> Option<Result<Date, chrono::format::ParseError>>;

    /// Parses the JSON value as `Time`.
    fn parse_time(&self) -> Option<Result<Time, chrono::format::ParseError>>;

    /// Parses the JSON value as `DateTime`.
    fn parse_date_time(&self) -> Option<Result<DateTime, chrono::format::ParseError>>;

    /// Parses the JSON value as `NaiveDateTime`.
    fn parse_naive_date_time(&self) -> Option<Result<NaiveDateTime, chrono::format::ParseError>>;

    /// Parses the JSON value as `Duration`.
    fn parse_duration(&self) -> Option<Result<Duration, datetime::ParseDurationError>>;

    /// Returns a pretty-printed String of JSON.
    fn to_string_pretty(&self) -> String;

    /// Returns a unquoted String of JSON.
    fn to_string_unquoted(&self) -> String;

    /// Attempts to convert the JSON value to the CSV bytes.
    fn to_csv(&self, buffer: Vec<u8>) -> Result<Vec<u8>, csv::Error>;

    /// Attempts to convert the JSON value to the JSON Lines bytes.
    fn to_jsonlines(&self, buffer: Vec<u8>) -> Result<Vec<u8>, serde_json::Error>;

    /// Attempts to deserialize the JSON value as an instance of type `T`.
    fn deserialize<T: DeserializeOwned>(self) -> Result<T, serde_json::Error>;

    /// Converts `self` into a map array.
    fn into_map_array(self) -> Vec<Map>;

    /// Converts `self` into a map option.
    fn into_map_opt(self) -> Option<Map>;
}

impl JsonValueExt for JsonValue {
    #[inline]
    fn is_ignorable(&self) -> bool {
        match self {
            JsonValue::Null => true,
            JsonValue::String(s) => s.is_empty(),
            JsonValue::Array(vec) => vec.is_empty(),
            JsonValue::Object(map) => map.is_empty(),
            _ => false,
        }
    }

    #[inline]
    fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|i| u8::try_from(i).ok())
    }

    #[inline]
    fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|i| u16::try_from(i).ok())
    }

    #[inline]
    fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|i| u32::try_from(i).ok())
    }

    #[inline]
    fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|i| usize::try_from(i).ok())
    }

    #[inline]
    fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|i| i8::try_from(i).ok())
    }

    #[inline]
    fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|i| i16::try_from(i).ok())
    }

    #[inline]
    fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|i| i32::try_from(i).ok())
    }

    #[inline]
    fn as_isize(&self) -> Option<isize> {
        self.as_i64().and_then(|i| isize::try_from(i).ok())
    }

    #[inline]
    fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|f| f as f32)
    }

    #[inline]
    fn as_str_array(&self) -> Option<Vec<&str>> {
        self.as_array()
            .map(|values| values.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
    }

    #[inline]
    fn as_map_array(&self) -> Option<Vec<&Map>> {
        self.as_array().map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_object())
                .collect::<Vec<_>>()
        })
    }

    #[inline]
    fn as_uuid(&self) -> Option<Uuid> {
        self.as_str().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn as_date(&self) -> Option<Date> {
        self.as_str().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn as_time(&self) -> Option<Time> {
        self.as_str().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn as_date_time(&self) -> Option<DateTime> {
        self.as_str().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn as_naive_date_time(&self) -> Option<NaiveDateTime> {
        self.as_str().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn as_duration(&self) -> Option<Duration> {
        self.as_str().and_then(|s| datetime::parse_duration(s).ok())
    }

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

    fn parse_i8(&self) -> Option<Result<i8, ParseIntError>> {
        self.as_i64()
            .and_then(|i| i8::try_from(i).ok())
            .map(Ok)
            .or_else(|| self.as_str().map(|s| s.parse()))
    }

    fn parse_i16(&self) -> Option<Result<i16, ParseIntError>> {
        self.as_i64()
            .and_then(|i| i16::try_from(i).ok())
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

    fn parse_isize(&self) -> Option<Result<isize, ParseIntError>> {
        self.as_i64()
            .and_then(|i| isize::try_from(i).ok())
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

    fn parse_array<T: FromStr>(&self) -> Option<Result<Vec<T>, <T as FromStr>::Err>> {
        let values = match &self {
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

    fn parse_str_array(&self) -> Option<Vec<&str>> {
        let values = match &self {
            JsonValue::String(s) => Some(helper::parse_str_array(s, ',')),
            JsonValue::Array(vec) => Some(vec.iter().filter_map(|v| v.as_str()).collect()),
            _ => None,
        };
        let vec = values?.iter().map(|s| s.trim()).collect::<Vec<_>>();
        (!vec.is_empty()).then_some(vec)
    }

    fn parse_uuid(&self) -> Option<Result<Uuid, uuid::Error>> {
        self.as_str()
            .map(|s| s.trim_start_matches("urn:uuid:"))
            .filter(|s| !s.chars().all(|c| c == '0' || c == '-'))
            .map(|s| s.parse())
    }

    #[inline]
    fn parse_decimal(&self) -> Option<Result<Decimal, rust_decimal::Error>> {
        self.as_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_date(&self) -> Option<Result<Date, chrono::format::ParseError>> {
        self.as_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_time(&self) -> Option<Result<Time, chrono::format::ParseError>> {
        self.as_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_date_time(&self) -> Option<Result<DateTime, chrono::format::ParseError>> {
        self.as_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_naive_date_time(&self) -> Option<Result<NaiveDateTime, chrono::format::ParseError>> {
        self.as_str().map(|s| s.parse())
    }

    #[inline]
    fn parse_duration(&self) -> Option<Result<Duration, datetime::ParseDurationError>> {
        self.as_str().map(datetime::parse_duration)
    }

    #[inline]
    fn to_string_pretty(&self) -> String {
        format!("{self:#}")
    }

    #[inline]
    fn to_string_unquoted(&self) -> String {
        self.as_str()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| self.to_string())
    }

    fn to_csv(&self, buffer: Vec<u8>) -> Result<Vec<u8>, csv::Error> {
        match &self {
            JsonValue::Array(vec) => {
                let mut wtr = Writer::from_writer(buffer);
                let mut headers = Vec::new();
                if let Some(JsonValue::Object(map)) = vec.first() {
                    for key in map.keys() {
                        headers.push(key.to_owned());
                    }
                }
                wtr.write_record(&headers)?;

                let num_fields = headers.len();
                let buffer_size = num_fields * 8;
                for value in vec {
                    if let JsonValue::Object(map) = value {
                        let mut record = ByteRecord::with_capacity(buffer_size, num_fields);
                        for field in headers.iter() {
                            let value = map.parse_string(field).unwrap_or("".into());
                            record.push_field(value.as_ref().as_bytes());
                        }
                        wtr.write_byte_record(&record)?;
                    }
                }
                wtr.flush()?;
                wtr.into_inner().map_err(|err| err.into_error().into())
            }
            JsonValue::Object(map) => {
                let mut wtr = Writer::from_writer(buffer);
                let mut headers = Vec::new();
                for key in map.keys() {
                    headers.push(key.to_owned());
                }
                wtr.write_record(&headers)?;

                let num_fields = headers.len();
                let buffer_size = num_fields * 8;
                let mut record = ByteRecord::with_capacity(buffer_size, num_fields);
                for field in headers.iter() {
                    let value = map.parse_string(field).unwrap_or("".into());
                    record.push_field(value.as_ref().as_bytes());
                }
                wtr.write_byte_record(&record)?;
                wtr.flush()?;
                wtr.into_inner().map_err(|err| err.into_error().into())
            }
            _ => Err(io::Error::new(ErrorKind::InvalidData, "invalid JSON value for CSV").into()),
        }
    }

    fn to_jsonlines(&self, mut buffer: Vec<u8>) -> Result<Vec<u8>, serde_json::Error> {
        match &self {
            JsonValue::Array(vec) => {
                for value in vec {
                    let mut jsonline = serde_json::to_vec(&value)?;
                    buffer.append(&mut jsonline);
                    buffer.push(b'\n');
                }
                Ok(buffer)
            }
            _ => {
                let mut jsonline = serde_json::to_vec(&self)?;
                buffer.append(&mut jsonline);
                buffer.push(b'\n');
                Ok(buffer)
            }
        }
    }

    #[inline]
    fn deserialize<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self)
    }

    #[inline]
    fn into_map_array(self) -> Vec<Map> {
        match self {
            JsonValue::Array(vec) => vec
                .into_iter()
                .filter_map(|v| v.into_map_opt())
                .collect::<Vec<_>>(),
            JsonValue::Object(map) => vec![map],
            _ => vec![],
        }
    }

    #[inline]
    fn into_map_opt(self) -> Option<Map> {
        if let JsonValue::Object(map) = self {
            Some(map)
        } else {
            None
        }
    }
}
