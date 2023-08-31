//! ISO 8601 combined date and time with local time zone.

use crate::{AvroValue, JsonValue};
use chrono::{
    format::ParseError, Datelike, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime,
    SecondsFormat, TimeZone, Utc,
};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
    time::Duration,
};

mod duration;

pub use duration::{parse_duration, ParseDurationError};

/// Alias for [`chrono::DateTime<Local>`](chrono::DateTime).
type LocalDateTime = chrono::DateTime<Local>;

/// A wrapper type for [`chrono::DateTime<Local>`](chrono::DateTime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct DateTime(LocalDateTime);

impl DateTime {
    /// Returns a new instance which corresponds to the current date.
    #[inline]
    pub fn now() -> Self {
        Self(Local::now())
    }

    /// Returns the number of non-leap seconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn current_timestamp() -> i64 {
        Utc::now().timestamp()
    }

    /// Returns the number of non-leap milliseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn current_timestamp_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    /// Returns the number of non-leap microseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn current_timestamp_micros() -> i64 {
        Utc::now().timestamp_micros()
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

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap microseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp_micros(micros: i64) -> Self {
        let dt = NaiveDateTime::from_timestamp_micros(micros).unwrap_or_default();
        Self(Local.from_utc_datetime(&dt))
    }

    /// Returns the number of non-leap seconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    /// Returns the number of non-leap milliseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    /// Returns the number of non-leap microseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn timestamp_micros(&self) -> i64 {
        self.0.timestamp_micros()
    }

    /// Returns the difference in seconds between `self` and
    /// the same date-time as evaluated in the UTC time zone.
    #[inline]
    pub fn timezone_offset(&self) -> i32 {
        self.0.offset().local_minus_utc()
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

    /// Returns a UTC timestamp string.
    #[inline]
    pub fn to_utc_timestamp(&self) -> String {
        let datetime = self.0.with_timezone(&Utc);
        format!("{}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"))
    }

    /// Returns an RFC 2822 date and time string.
    #[inline]
    pub fn to_utc_string(&self) -> String {
        let datetime = self.0.with_timezone(&Utc);
        format!("{} GMT", datetime.to_rfc2822().trim_end_matches(" +0000"))
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with subseconds
    /// formatted as [`SecondsFormat::Millis`].
    #[inline]
    pub fn to_iso_string(&self) -> String {
        let datetime = self.0.with_timezone(&Utc);
        datetime.to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    /// Formats the combined date and time with the specified format string.
    /// See [`format::strftime`](chrono::format::strftime) for the supported escape sequences.
    #[inline]
    pub fn format(&self, fmt: &str) -> String {
        format!("{}", self.0.format(fmt))
    }

    /// Returns a date-only string in the format `%Y-%m-%d`.
    #[inline]
    pub fn format_date(&self) -> String {
        format!("{}", self.0.format("%Y-%m-%d"))
    }

    /// Returns a time-only string in the format `%H:%M:%S`.
    #[inline]
    pub fn format_time(&self) -> String {
        format!("{}", self.0.format("%H:%M:%S"))
    }

    /// Returns a date-time string in the format `%Y-%m-%d %H:%M:%S` with the `Local` time zone.
    pub fn format_local(&self) -> String {
        format!("{}", self.0.format("%Y-%m-%d %H:%M:%S"))
    }

    /// Returns a date-time string in the format `%Y-%m-%d %H:%M:%S` with the `Utc` time zone.
    #[inline]
    pub fn format_utc(&self) -> String {
        let datetime = self.0.with_timezone(&Utc);
        format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"))
    }

    /// Returns the amount of time elapsed from another datetime to this one,
    /// or zero duration if that datetime is later than this one.
    #[inline]
    pub fn duration_since(&self, earlier: DateTime) -> Duration {
        (self.0 - earlier.0).to_std().unwrap_or_default()
    }

    /// Returns the duration of time between `self` and `other`.
    #[inline]
    pub fn span_between(&self, other: DateTime) -> Duration {
        let timestamp = self.timestamp_micros();
        let current_timestamp = other.timestamp_micros();
        Duration::from_micros(current_timestamp.abs_diff(timestamp))
    }

    /// Returns the duration of time between `self` and `DateTime::now()`.
    #[inline]
    pub fn span_between_now(&self) -> Duration {
        let timestamp = self.timestamp_micros();
        let current_timestamp = Local::now().timestamp_micros();
        Duration::from_micros(current_timestamp.abs_diff(timestamp))
    }

    /// Returns the duration of time from `self` to `DateTime::now()`.
    pub fn span_before_now(&self) -> Option<Duration> {
        let timestamp = self.timestamp_micros();
        let current_timestamp = Local::now().timestamp_micros();
        if current_timestamp >= timestamp {
            u64::try_from(current_timestamp - timestamp)
                .ok()
                .map(Duration::from_micros)
        } else {
            None
        }
    }

    /// Returns the duration of time from `DateTime::now()` to `self`.
    pub fn span_after_now(&self) -> Option<Duration> {
        let timestamp = self.timestamp_micros();
        let current_timestamp = Local::now().timestamp_micros();
        if current_timestamp <= timestamp {
            u64::try_from(timestamp - current_timestamp)
                .ok()
                .map(Duration::from_micros)
        } else {
            None
        }
    }

    /// Returns the year number in the calendar date.
    #[inline]
    pub fn year(&self) -> i32 {
        self.0.year()
    }

    /// Returns the month number starting from 1.
    ///
    /// The return value ranges from 1 to 12.
    #[inline]
    pub fn month(&self) -> u32 {
        self.0.month()
    }

    /// Returns the day of month starting from 1.
    ///
    /// The return value ranges from 1 to 31. (The last day of month differs by months.)
    #[inline]
    pub fn day(&self) -> u32 {
        self.0.day()
    }

    /// Returns `true` if the current year is a leap year.
    #[inline]
    pub fn is_leap_year(&self) -> bool {
        let year = self.year();
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Returns the number of days in the current year.
    #[inline]
    pub fn days_in_current_year(&self) -> u32 {
        if self.is_leap_year() {
            366
        } else {
            365
        }
    }

    /// Returns the number of days in the current month.
    pub fn days_in_current_month(&self) -> u32 {
        let month = self.month();
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            _ => panic!("invalid month: {month}"),
        }
    }

    /// Returns the start of the current year.
    pub fn start_of_current_year(&self) -> Self {
        let year = self.year();
        let date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the end of the current year.
    pub fn end_of_current_year(&self) -> Self {
        let year = self.year();
        let dt = NaiveDate::from_ymd_opt(year, 12, 31)
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the start of the current month.
    pub fn start_of_current_month(&self) -> Self {
        let year = self.year();
        let month = self.month();
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the end of the current month.
    pub fn end_of_current_month(&self) -> Self {
        let year = self.year();
        let month = self.month();
        let day = self.days_in_current_month();
        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the start of the current day.
    pub fn start_of_current_day(&self) -> Self {
        let date = self.0.date_naive();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the end of the current day.
    pub fn end_of_current_day(&self) -> Self {
        let date = self.0.date_naive();
        let dt = date
            .and_hms_milli_opt(23, 59, 59, 1_000)
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the start of the year.
    pub fn start_of_year(year: i32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the end of the year.
    pub fn end_of_year(year: i32) -> Self {
        let dt = NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .and_then(|date| date.pred_opt())
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the start of the month.
    pub fn start_of_month(year: i32, month: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the end of the month.
    pub fn end_of_month(year: i32, month: u32) -> Self {
        let dt = NaiveDate::from_ymd_opt(year, month + 1, 1)
            .and_then(|date| date.pred_opt())
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the start of the day.
    pub fn start_of_day(year: i32, month: u32, day: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Returns the end of the month.
    pub fn end_of_day(year: i32, month: u32, day: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default();
        let dt = date
            .and_hms_milli_opt(23, 59, 59, 1_000)
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(dt, offset))
    }

    /// Adds a duration in months to the date part of the `DateTime`.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_add_months(self, months: u32) -> Option<Self> {
        self.0.checked_add_months(Months::new(months)).map(Self)
    }

    /// Subtracts a duration in months from the date part of the `DateTime`.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_sub_months(self, months: u32) -> Option<Self> {
        self.0.checked_sub_months(Months::new(months)).map(Self)
    }

    /// Adds a duration in days to the date part of the `DateTime`.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_add_days(self, days: u32) -> Option<Self> {
        self.0
            .checked_add_days(Days::new(u64::from(days)))
            .map(Self)
    }

    /// Subtracts a duration in days from the date part of the `DateTime`.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_sub_days(self, days: u32) -> Option<Self> {
        self.0
            .checked_sub_days(Days::new(u64::from(days)))
            .map(Self)
    }
}

impl Default for DateTime {
    /// Returns an instance which corresponds to **the current date and time**.
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for DateTime {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S%.6f %z"))
    }
}

impl Serialize for DateTime {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_utc_timestamp())
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let length = s.len();
        if length == 10 {
            let date = s.parse::<NaiveDate>()?;
            let dt = NaiveDateTime::new(date, NaiveTime::default());
            let offset = Local.offset_from_utc_datetime(&dt);
            Ok(LocalDateTime::from_naive_utc_and_offset(dt, offset).into())
        } else if length == 19 {
            let dt = s.parse::<NaiveDateTime>()?;
            let offset = Local.offset_from_utc_datetime(&dt);
            Ok(LocalDateTime::from_naive_utc_and_offset(dt, offset).into())
        } else {
            LocalDateTime::from_str(s).map(Self)
        }
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

#[cfg(test)]
mod tests {
    use super::DateTime;

    #[test]
    fn it_parses_datetime() {
        assert!("2023-12-31".parse::<DateTime>().is_ok());
        assert!("2023-12-31T18:00:00".parse::<DateTime>().is_ok());
        assert!("2023-07-13T02:16:33.449Z".parse::<DateTime>().is_ok());
        assert!("2023-06-10 05:17:23.713071 +0800"
            .parse::<DateTime>()
            .is_ok());
    }
}
