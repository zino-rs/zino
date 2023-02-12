use crate::{Map, Record};
use apache_avro::{types::Value, Error};
use std::{collections::HashMap, mem};

/// Extension trait for [`Record`](crate::Record).
pub trait AvroRecordExt {
    /// Extracts the string corresponding to the field.
    fn get_str(&self, field: &str) -> Option<&str>;

    /// Inserts or updates a field/value pair into the record.
    /// If the record did have this field, the value is updated and the old value is returned,
    /// otherwise `None` is returned.
    fn upsert(&mut self, field: impl Into<String>, value: impl Into<Value>) -> Option<Value>;

    /// Returns `true` if the record contains a value for the field.
    fn contains_field(&self, field: &str) -> bool;

    /// Searches for the field and returns its value.
    fn find(&self, field: &str) -> Option<&Value>;

    /// Searches for the field and returns its index.
    fn position(&self, field: &str) -> Option<usize>;

    /// Converts `self` to an Avro map value.
    fn into_avro_map(self) -> Value;

    /// Consumes `self` and attempts to construct a json object.
    fn try_into_map(self) -> Result<Map, Error>;
}

impl AvroRecordExt for Record {
    fn get_str(&self, field: &str) -> Option<&str> {
        self.find(field).and_then(|v| {
            if let Value::String(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        })
    }

    fn upsert(&mut self, field: impl Into<String>, value: impl Into<Value>) -> Option<Value> {
        let field = field.into();
        let key = field.as_str();
        if let Some(index) = self.iter().position(|(field, _)| field == key) {
            Some(mem::replace(&mut self[index].1, value.into()))
        } else {
            self.push((field, value.into()));
            None
        }
    }

    #[inline]
    fn contains_field(&self, field: &str) -> bool {
        self.iter().any(|(key, _)| field == key)
    }

    #[inline]
    fn find(&self, field: &str) -> Option<&Value> {
        self.iter()
            .find_map(|(key, value)| (field == key).then_some(value))
    }

    #[inline]
    fn position(&self, field: &str) -> Option<usize> {
        self.iter().position(|(key, _)| field == key)
    }

    fn into_avro_map(self) -> Value {
        let mut map = HashMap::new();
        for (field, value) in self.into_iter() {
            map.insert(field, value);
        }
        Value::Map(map)
    }

    fn try_into_map(self) -> Result<Map, Error> {
        let mut map = Map::new();
        for (field, value) in self.into_iter() {
            map.insert(field, value.try_into()?);
        }
        Ok(map)
    }
}
