use crate::{Map, Record};
use serde_json::Value;

/// Extension trait for [`Map`](crate::Map).
pub trait JsonObjectExt {
    /// Extracts the boolean value corresponding to the key.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Extracts the integer value corresponding to the key and
    /// represents it as `i32` if possible.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Extracts the integer value corresponding to the key.
    fn get_i64(&self, key: &str) -> Option<i64>;

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

    /// Extracts the float value corresponding to the key and
    /// represents it as `f32` if possible.
    fn get_f32(&self, key: &str) -> Option<f32>;

    /// Extracts the float value corresponding to the key.
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Extracts the array value corresponding to the key.
    fn get_array(&self, key: &str) -> Option<&Vec<Value>>;

    /// Extracts the object value corresponding to the key.
    fn get_object(&self, key: &str) -> Option<&Map>;

    /// Inserts or updates a key/value pair into the map.
    /// If the map did have this key present, the value is updated and the old value is returned,
    /// otherwise `None` is returned.
    fn upsert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> Option<Value>;

    /// Consumes `self` and constructs an Avro record value.
    fn into_avro_record(self) -> Record;

    /// Creates a new instance with the entry.
    fn from_entry(key: impl Into<String>, value: impl Into<Value>) -> Self;

    /// Creates a new instance with a single key `entry`.
    fn data_entry(value: impl Into<Value>) -> Self;

    /// Creates a new instance with a single key `entries`.
    fn data_entries(value: Vec<Map>) -> Self;
}

impl JsonObjectExt for Map {
    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    #[inline]
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key)
            .and_then(|v| v.as_u64())
            .and_then(|i| i32::try_from(i).ok())
    }

    #[inline]
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
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
    fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.get(key).and_then(|v| v.as_array())
    }

    #[inline]
    fn get_object(&self, key: &str) -> Option<&Map> {
        self.get(key).and_then(|v| v.as_object())
    }

    #[inline]
    fn upsert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> Option<Value> {
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
    fn from_entry(key: impl Into<String>, value: impl Into<Value>) -> Self {
        let mut map = Map::with_capacity(1);
        map.insert(key.into(), value.into());
        map
    }

    #[inline]
    fn data_entry(value: impl Into<Value>) -> Self {
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
