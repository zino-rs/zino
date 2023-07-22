use crate::{extension::JsonObjectExt, JsonValue, Map};
use csv::{ByteRecord, Writer};
use std::{
    borrow::Cow,
    io::{self, ErrorKind},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, ParseBoolError},
};

/// Extension trait for [`serde_json::Value`].
pub trait JsonValueExt {
    /// Returns `true` if the JSON value can be ignorable.
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

    /// If the `Value` is an integer, represent it as `i32` if possible.
    /// Returns `None` otherwise.
    fn as_i32(&self) -> Option<i32>;

    /// If the `Value` is a float, represent it as `f32` if possible.
    /// Returns `None` otherwise.
    fn as_f32(&self) -> Option<f32>;

    /// If the `Value` is an array of strings, returns the associated vector.
    /// Returns `None` otherwise.
    fn as_str_array(&self) -> Option<Vec<&str>>;

    /// If the `Value` is an array of maps, returns the associated vector.
    /// Returns `None` otherwise.
    fn as_map_array(&self) -> Option<Vec<&Map>>;

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

    /// Parses the JSON value as `i32`.
    fn parse_i32(&self) -> Option<Result<i32, ParseIntError>>;

    /// Parses the JSON value as `i64`.
    fn parse_i64(&self) -> Option<Result<i64, ParseIntError>>;

    /// Parses the JSON value as `f32`.
    fn parse_f32(&self) -> Option<Result<f32, ParseFloatError>>;

    /// Parses the JSON value as `f64`.
    fn parse_f64(&self) -> Option<Result<f64, ParseFloatError>>;

    /// Parses the JSON value as `Cow<'_, str>`.
    /// If the str is empty, it also returns `None`.
    fn parse_string(&self) -> Option<Cow<'_, str>>;

    /// Parses the JSON value as `Vec<T>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_array<T: FromStr>(&self) -> Option<Vec<T>>;

    /// Parses the JSON value as `Vec<&str>`.
    /// If the vec is empty, it also returns `None`.
    fn parse_str_array(&self) -> Option<Vec<&str>>;

    /// Attempts to convert the JSON value to the CSV bytes.
    fn to_csv(&self, buffer: Vec<u8>) -> Result<Vec<u8>, csv::Error>;

    /// Attempts to convert the JSON value to the JSON Lines bytes.
    fn to_jsonlines(&self, buffer: Vec<u8>) -> Result<Vec<u8>, serde_json::Error>;

    /// Attempts to convert the JSON value to the MsgPack bytes.
    fn to_msgpack(&self, buffer: Vec<u8>) -> Result<Vec<u8>, rmp_serde::encode::Error>;

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
    fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|i| i32::try_from(i).ok())
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
            JsonValue::Array(vec) => Some(vec.iter().filter_map(|v| v.as_str()).collect()),
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
            JsonValue::Array(vec) => Some(vec.iter().filter_map(|v| v.as_str()).collect()),
            _ => None,
        };
        let vec = values?.iter().map(|s| s.trim()).collect::<Vec<_>>();
        (!vec.is_empty()).then_some(vec)
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
    fn to_msgpack(&self, mut buffer: Vec<u8>) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::encode::write(&mut buffer, &self)?;
        Ok(buffer)
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
