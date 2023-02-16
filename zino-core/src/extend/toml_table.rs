use crate::datetime;
use std::time::Duration;
use toml::{Table, Value};

/// Extension trait for [`Table`](toml::Table).
pub trait TomlTableExt {
    /// Extracts the boolean value corresponding to the key.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Extracts the integer value corresponding to the key.
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
    fn get_f64(&self, key: &str) -> Option<f64>;

    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Extracts the array value corresponding to the key.
    fn get_array(&self, key: &str) -> Option<&Vec<Value>>;

    /// Extracts the table value corresponding to the key.
    fn get_table(&self, key: &str) -> Option<&Table>;

    /// Extracts the string corresponding to the key
    /// and parses it as `Duration`.
    fn get_duration(&self, key: &str) -> Option<Duration>;
}

impl TomlTableExt for Table {
    #[inline]
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    #[inline]
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_integer())
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
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_float())
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
    fn get_table(&self, key: &str) -> Option<&Table> {
        self.get(key).and_then(|v| v.as_table())
    }

    fn get_duration(&self, key: &str) -> Option<Duration> {
        self.get_str(key)
            .and_then(|s| datetime::parse_duration(s).ok())
    }
}
