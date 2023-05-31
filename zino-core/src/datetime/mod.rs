//! ISO 8601 combined date and time with local time zone.

use apache_avro::types::Value as AvroValue;
use chrono::{format::ParseError, Local, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    fmt,
    ops::{Add, AddAssign, Deref, Sub, SubAssign},
    str::FromStr,
    time::Duration,
};

mod duration;

pub use duration::{parse_duration, ParseDurationError};

/// Alias for [`chrono::DateTime<Local>`](chrono::DateTime).
type LocalDateTime = chrono::DateTime<Local>;

/// A wrapper type for [`chrono::DateTime<Local>`](chrono::DateTime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DateTime(LocalDateTime);

impl DateTime {
    /// Returns a new instance which corresponds to the current date.
    #[inline]
    pub fn now() -> Self {
        Self(Local::now())
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap seconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp(secs: i64) -> Self {
        let dt = NaiveDateTime::from_timestamp_opt(secs, 0).unwrap_or_default();
        Self(Local.from_utc_datetime(&dt))
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap milliseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp_millis(millis: i64) -> Self {
        let dt = NaiveDateTime::from_timestamp_millis(millis).unwrap_or_default();
        Self(Local.from_utc_datetime(&dt))
    }

    /// Parses an RFC 2822 date and time.
    #[inline]
    pub fn parse_utc_str(s: &str) -> Result<Self, ParseError> {
        let datetime = chrono::DateTime::parse_from_rfc2822(s)?;
        Ok(Self(datetime.with_timezone(&Local)))
    }

    /// Parses an RFC 3339 and ISO 8601 date and time.
    #[inline]
    pub fn parse_iso_str(s: &str) -> Result<Self, ParseError> {
        let datetime = chrono::DateTime::parse_from_rfc3339(s)?;
        Ok(Self(datetime.with_timezone(&Local)))
    }

    /// Parses a string with the specified format string.
    /// See [`format::strftime`](chrono::format::strftime) for the supported escape sequences.
    #[inline]
    pub fn parse_from_str(s: &str, fmt: &str) -> Result<Self, ParseError> {
        let datetime = chrono::DateTime::parse_from_str(s, fmt)?;
        Ok(Self(datetime.with_timezone(&Local)))
    }

    /// Returns an RFC 2822 date and time string.
    #[inline]
    pub fn to_utc_string(&self) -> String {
        let datetime = self.0.with_timezone(&Utc);
        format!("{} GMT", datetime.to_rfc2822().trim_end_matches(" +0000"))
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with subseconds
    /// formatted as [`SecondsFormat::Millis`](chrono::SecondsFormat::Millis).
    #[inline]
    pub fn to_iso_string(&self) -> String {
        let datetime = self.0.with_timezone(&Utc);
        datetime.to_rfc3339_opts(SecondsFormat::Millis, true)
    }
}

impl fmt::Display for DateTime {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S%.f%z"))
    }
}

impl Default for DateTime {
    /// Returns an instance which corresponds to **the current date and time**.
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl Deref for DateTime {
    type Target = LocalDateTime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<LocalDateTime> for DateTime {
    #[inline]
    fn from(dt: LocalDateTime) -> Self {
        Self(dt)
    }
}

impl From<DateTime> for LocalDateTime {
    #[inline]
    fn from(dt: DateTime) -> Self {
        dt.0
    }
}

impl From<DateTime> for AvroValue {
    #[inline]
    fn from(dt: DateTime) -> Self {
        AvroValue::String(dt.to_string())
    }
}

impl From<DateTime> for JsonValue {
    #[inline]
    fn from(dt: DateTime) -> Self {
        JsonValue::String(dt.to_string())
    }
}

impl FromStr for DateTime {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LocalDateTime::from_str(s).map(Self)
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self {
        let duration = chrono::Duration::from_std(rhs).expect("Duration value is out of range");
        let datetime = self
            .0
            .checked_add_signed(duration)
            .expect("`DateTime + Duration` overflowed");
        Self(datetime)
    }
}

impl AddAssign<Duration> for DateTime {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Duration) -> Self {
        let duration = chrono::Duration::from_std(rhs).expect("Duration value is out of range");
        let datetime = self
            .0
            .checked_sub_signed(duration)
            .expect("`DateTime - Duration` overflowed");
        Self(datetime)
    }
}

impl SubAssign<Duration> for DateTime {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
