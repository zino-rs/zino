use crate::{error::Error, AvroValue, JsonValue};
use chrono::{format::ParseError, Local, NaiveTime, Timelike};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
    time::Duration,
};

/// A wrapper type for [`chrono::NaiveTime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct Time(NaiveTime);

impl Time {
    /// Attempts to create a new instance.
    #[inline]
    pub fn try_new(hour: u32, minute: u32, second: u32) -> Result<Self, Error> {
        NaiveTime::from_hms_opt(hour, minute, second)
            .map(Self)
            .ok_or_else(|| {
                let message = format!(
                    "fail to create a time from hour: `{hour}`, minute: `{minute}`, second: `{second}`"
                );
                Error::new(message)
            })
    }

    /// Returns a new instance which corresponds to the current time.
    #[inline]
    pub fn now() -> Self {
        Self(Local::now().time())
    }

    /// Returns a new instance which corresponds to the midnight.
    #[inline]
    pub fn midnight() -> Self {
        Self(NaiveTime::default())
    }

    /// Returns the number of non-leap seconds past the last midnight.
    #[inline]
    pub fn num_secs_from_midnight(&self) -> u32 {
        self.0.num_seconds_from_midnight()
    }

    /// Returns the number of non-leap milliseconds past the last midnight.
    #[inline]
    pub fn num_millis_from_midnight(&self) -> u32 {
        self.0.num_seconds_from_midnight() * 1000 + self.0.nanosecond() / 1_000_000
    }

    /// Returns the number of non-leap microseconds past the last midnight.
    #[inline]
    pub fn num_micros_from_midnight(&self) -> u32 {
        self.0.num_seconds_from_midnight() * 1_000_000 + self.0.nanosecond() / 1000
    }

    /// Formats the time with the specified format string.
    /// See [`format::strftime`](chrono::format::strftime) for the supported escape sequences.
    #[inline]
    pub fn format(&self, fmt: &str) -> String {
        format!("{}", self.0.format(fmt))
    }

    /// Returns the amount of time elapsed from another time to this one,
    /// or zero duration if that time is later than this one.
    #[inline]
    pub fn duration_since(&self, earlier: Time) -> Duration {
        (self.0 - earlier.0).to_std().unwrap_or_default()
    }

    /// Returns the duration of time between `self` and `other`.
    #[inline]
    pub fn span_between(&self, other: Time) -> Duration {
        let duration = if self > &other {
            self.0 - other.0
        } else {
            other.0 - self.0
        };
        duration.to_std().unwrap_or_default()
    }

    /// Returns the duration of time between `self` and `Time::now()`.
    #[inline]
    pub fn span_between_now(&self) -> Duration {
        self.span_between(Self::now())
    }

    /// Returns the duration of time from `self` to `Time::now()`.
    #[inline]
    pub fn span_before_now(&self) -> Option<Duration> {
        let current = Self::now();
        if self <= &current {
            (current.0 - self.0).to_std().ok()
        } else {
            None
        }
    }

    /// Returns the duration of time from `Time::now()` to `self`.
    #[inline]
    pub fn span_after_now(&self) -> Option<Duration> {
        let current = Self::now();
        if self >= &current {
            (self.0 - current.0).to_std().ok()
        } else {
            None
        }
    }

    /// Returns the hour number from 0 to 23.
    #[inline]
    pub fn hour(&self) -> u32 {
        self.0.hour()
    }

    /// Returns the minute number from 0 to 59.
    #[inline]
    pub fn minute(&self) -> u32 {
        self.0.minute()
    }

    /// Returns the second number from 0 to 59.
    #[inline]
    pub fn second(&self) -> u32 {
        self.0.second()
    }
}

impl Default for Time {
    /// Returns an instance which corresponds to **the current time**.
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Time {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%H:%M:%S%.f"))
    }
}

impl Serialize for Time {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<NaiveTime> for Time {
    #[inline]
    fn from(t: NaiveTime) -> Self {
        Self(t)
    }
}

impl From<Time> for NaiveTime {
    #[inline]
    fn from(t: Time) -> Self {
        t.0
    }
}

impl From<Time> for AvroValue {
    #[inline]
    fn from(t: Time) -> Self {
        let micros = t.num_micros_from_midnight();
        AvroValue::TimeMicros(micros.into())
    }
}

impl From<Time> for JsonValue {
    #[inline]
    fn from(t: Time) -> Self {
        JsonValue::String(t.to_string())
    }
}

impl FromStr for Time {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<NaiveTime>().map(Self)
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self {
        let duration = chrono::Duration::from_std(rhs).expect("Duration value is out of range");
        Self(self.0 + duration)
    }
}

impl AddAssign<Duration> for Time {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for Time {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Duration) -> Self {
        let duration = chrono::Duration::from_std(rhs).expect("Duration value is out of range");
        Self(self.0 - duration)
    }
}

impl SubAssign<Duration> for Time {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
