use crate::{AvroValue, Map, Record};
use apache_avro::{Error, Schema, Writer};
use std::{collections::HashMap, io::Write, mem};

/// Extension trait for [`Record`](crate::Record).
pub trait AvroRecordExt {
    /// Extracts the boolean value corresponding to the key.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Extracts the integer value corresponding to the key.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Extracts the `Long` integer value corresponding to the key.
    fn get_i64(&self, key: &str) -> Option<i64>;

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

    /// Extracts the float value corresponding to the key.
    fn get_f32(&self, key: &str) -> Option<f32>;

    /// Extracts the float value corresponding to the key.
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Extracts the bytes corresponding to the key.
    fn get_bytes(&self, key: &str) -> Option<&[u8]>;

    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Returns `true` if the record contains a value for the key.
    fn contains_key(&self, key: &str) -> bool;

    /// Searches for the key and returns its value.
    fn find(&self, key: &str) -> Option<&AvroValue>;

    /// Searches for the key and returns its index.
    fn position(&self, key: &str) -> Option<usize>;

    /// Inserts or updates a  pair into the record.
    /// If the record did have this key, the value is updated and the old value is returned,
    /// otherwise `None` is returned.
    fn upsert(&mut self, key: impl Into<String>, value: impl Into<AvroValue>) -> Option<AvroValue>;

    /// Flushes the content appended to a writer with the given schema.
    /// Returns the number of bytes written.
    fn flush_to_writer<W: Write>(self, schema: &Schema, writer: W) -> Result<usize, Error>;

    /// Converts `self` to an Avro map.
    fn into_avro_map(self) -> HashMap<String, AvroValue>;

    /// Consumes `self` and attempts to construct a json object.
    fn try_into_map(self) -> Result<Map, Error>;

    /// Creates a new instance with the entry.
    fn from_entry(key: impl Into<String>, value: impl Into<AvroValue>) -> Self;
}

impl AvroRecordExt for Record {
    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.find(key).and_then(|v| {
            if let AvroValue::Boolean(b) = v {
                Some(*b)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.find(key).and_then(|v| {
            if let AvroValue::Int(i) = v {
                Some(*i)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.find(key).and_then(|v| {
            if let AvroValue::Long(i) = v {
                Some(*i)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_u16(&self, key: &str) -> Option<u16> {
        self.find(key).and_then(|v| {
            if let AvroValue::Int(i) = v {
                u16::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_u32(&self, key: &str) -> Option<u32> {
        self.find(key).and_then(|v| {
            if let AvroValue::Int(i) = v {
                u32::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_u64(&self, key: &str) -> Option<u64> {
        self.find(key).and_then(|v| {
            if let AvroValue::Long(i) = v {
                u64::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_usize(&self, key: &str) -> Option<usize> {
        self.find(key).and_then(|v| {
            if let AvroValue::Long(i) = v {
                usize::try_from(*i).ok()
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_f32(&self, key: &str) -> Option<f32> {
        self.find(key).and_then(|v| {
            if let AvroValue::Float(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.find(key).and_then(|v| {
            if let AvroValue::Double(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    fn get_bytes(&self, key: &str) -> Option<&[u8]> {
        self.find(key).and_then(|v| {
            if let AvroValue::Bytes(vec) = v {
                Some(vec.as_slice())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.find(key).and_then(|v| {
            if let AvroValue::String(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        })
    }

    #[inline]
    fn contains_key(&self, key: &str) -> bool {
        self.iter().any(|(field, _)| field == key)
    }

    #[inline]
    fn find(&self, key: &str) -> Option<&AvroValue> {
        self.iter()
            .find_map(|(field, value)| (field == key).then_some(value))
    }

    #[inline]
    fn position(&self, key: &str) -> Option<usize> {
        self.iter().position(|(field, _)| field == key)
    }

    fn upsert(&mut self, key: impl Into<String>, value: impl Into<AvroValue>) -> Option<AvroValue> {
        let field = key.into();
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
        writer.append(AvroValue::Record(self))?;
        writer.flush()
    }

    fn into_avro_map(self) -> HashMap<String, AvroValue> {
        let mut map = HashMap::with_capacity(self.len());
        for (key, value) in self.into_iter() {
            map.insert(key, value);
        }
        map
    }

    fn try_into_map(self) -> Result<Map, Error> {
        let mut map = Map::new();
        for (key, value) in self.into_iter() {
            map.insert(key, value.try_into()?);
        }
        Ok(map)
    }

    #[inline]
    fn from_entry(key: impl Into<String>, value: impl Into<AvroValue>) -> Self {
        vec![(key.into(), value.into())]
    }
}
