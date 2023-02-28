use crate::{Map, Record};
use apache_avro::{types::Value, Error, Schema, Writer};
use std::{collections::HashMap, io::Write, mem};

/// Extension trait for [`Record`](crate::Record).
pub trait AvroRecordExt {
    /// Extracts the boolean value corresponding to the field.
    fn get_bool(&self, field: &str) -> Option<bool>;

    /// Extracts the integer value corresponding to the field.
    fn get_i32(&self, field: &str) -> Option<i32>;

    /// Extracts the `Long` integer value corresponding to the field.
    fn get_i64(&self, field: &str) -> Option<i64>;

    /// Extracts the integer value corresponding to the field and
    /// represents it as `u16` if possible.
    fn get_u16(&self, field: &str) -> Option<u16>;

    /// Extracts the integer value corresponding to the field and
    /// represents it as `u32` if possible.
    fn get_u32(&self, field: &str) -> Option<u32>;

    /// Extracts the integer value corresponding to the field and
    /// represents it as `u64` if possible.
    fn get_u64(&self, field: &str) -> Option<u64>;

    /// Extracts the integer value corresponding to the field and
    /// represents it as `usize` if possible.
    fn get_usize(&self, field: &str) -> Option<usize>;

    /// Extracts the float value corresponding to the field.
    fn get_f32(&self, field: &str) -> Option<f32>;

    /// Extracts the float value corresponding to the field.
    fn get_f64(&self, field: &str) -> Option<f64>;

    /// Extracts the bytes corresponding to the field.
    fn get_bytes(&self, field: &str) -> Option<&[u8]>;

    /// Extracts the string corresponding to the field.
    fn get_str(&self, field: &str) -> Option<&str>;

    /// Returns `true` if the record contains a value for the field.
    fn contains_field(&self, field: &str) -> bool;

    /// Searches for the field and returns its value.
    fn find(&self, field: &str) -> Option<&Value>;

    /// Searches for the field and returns its index.
    fn position(&self, field: &str) -> Option<usize>;

    /// Inserts or updates a field/value pair into the record.
    /// If the record did have this field, the value is updated and the old value is returned,
    /// otherwise `None` is returned.
    fn upsert(&mut self, field: impl Into<String>, value: impl Into<Value>) -> Option<Value>;

    /// Flushes the content appended to a writer with the given schema.
    /// Returns the number of bytes written.
    fn flush_to_writer<W: Write>(self, schema: &Schema, writer: W) -> Result<usize, Error>;

    /// Converts `self` to an Avro map.
    fn into_avro_map(self) -> HashMap<String, Value>;

    /// Consumes `self` and attempts to construct a json object.
    fn try_into_map(self) -> Result<Map, Error>;
}

impl AvroRecordExt for Record {
    #[inline]
    fn get_bool(&self, field: &str) -> Option<bool> {
        self.find(field).and_then(|v| {
            if let Value::Boolean(b) = v {
                Some(*b)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i32(&self, field: &str) -> Option<i32> {
        self.find(field).and_then(|v| {
            if let Value::Int(i) = v {
                Some(*i)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i64(&self, field: &str) -> Option<i64> {
        self.find(field).and_then(|v| {
            if let Value::Long(i) = v {
                Some(*i)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_u16(&self, field: &str) -> Option<u16> {
        self.find(field).and_then(|v| {
            if let Value::Int(i) = v {
                u16::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_u32(&self, field: &str) -> Option<u32> {
        self.find(field).and_then(|v| {
            if let Value::Int(i) = v {
                u32::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_u64(&self, field: &str) -> Option<u64> {
        self.find(field).and_then(|v| {
            if let Value::Long(i) = v {
                u64::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_usize(&self, field: &str) -> Option<usize> {
        self.find(field).and_then(|v| {
            if let Value::Long(i) = v {
                usize::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_f32(&self, field: &str) -> Option<f32> {
        self.find(field).and_then(|v| {
            if let Value::Float(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_f64(&self, field: &str) -> Option<f64> {
        self.find(field).and_then(|v| {
            if let Value::Double(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    fn get_bytes(&self, field: &str) -> Option<&[u8]> {
        self.find(field).and_then(|v| {
            if let Value::Bytes(vec) = v {
                Some(vec.as_slice())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_str(&self, field: &str) -> Option<&str> {
        self.find(field).and_then(|v| {
            if let Value::String(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        })
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

    fn flush_to_writer<W: Write>(self, schema: &Schema, writer: W) -> Result<usize, Error> {
        let mut writer = Writer::new(schema, writer);
        writer.append(Value::Record(self))?;
        writer.flush()
    }

    fn into_avro_map(self) -> HashMap<String, Value> {
        let mut map = HashMap::with_capacity(self.len());
        for (field, value) in self.into_iter() {
            map.insert(field, value);
        }
        map
    }

    fn try_into_map(self) -> Result<Map, Error> {
        let mut map = Map::with_capacity(self.len());
        for (field, value) in self.into_iter() {
            map.insert(field, value.try_into()?);
        }
        Ok(map)
    }
}
