use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
    time::Duration,
};
use time::{
    error::Parse,
    format_description::well_known::{Rfc2822, Rfc3339},
    OffsetDateTime, UtcOffset,
};

/// ISO 8601 combined date and time with local time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DateTime(OffsetDateTime);

impl DateTime {
    /// Returns a new instance which corresponds to the current date.
    #[inline]
    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap seconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp(secs: i64) -> Self {
        Self(OffsetDateTime::from_unix_timestamp(secs).unwrap())
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC.
    #[inline]
    pub fn timestamp(&self) -> i64 {
        self.0.unix_timestamp()
    }

    /// Parses an RFC 2822 date and time.
    #[inline]
    pub fn parse_utc_str(s: &str) -> Result<Self, Parse> {
        let datetime = OffsetDateTime::parse(s, &Rfc2822)?;
        Ok(Self(datetime))
    }

    /// Returns an RFC 2822 date and time string.
    #[inline]
    pub fn to_utc_string(&self) -> String {
        let datetime = self.0.to_offset(UtcOffset::UTC).format(&Rfc2822).unwrap();
        format!("{} GMT", datetime.trim_end_matches(" +0000"))
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self(OffsetDateTime::now_utc())
    }
}

impl From<DateTime> for OffsetDateTime {
    fn from(t: DateTime) -> Self {
        t.0
    }
}

impl FromStr for DateTime {
    type Err = Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OffsetDateTime::parse(s, &Rfc3339).map(Self)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self {
        Self(self.0 + rhs)
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
        Self(self.0 - rhs)
    }
}

impl SubAssign<Duration> for DateTime {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
