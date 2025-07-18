//! ISO 8601 combined date and time with local time zone.

use crate::{AvroValue, JsonValue};
use chrono::{
    Datelike, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, TimeZone,
    Timelike, Utc, Weekday, format::ParseError,
};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
    time::Duration,
};
use uuid::{NoContext, Timestamp};

mod date;
mod duration;
mod time;

pub use date::Date;
pub use duration::{ParseDurationError, parse_duration};
pub use time::Time;

/// Alias for [`chrono::DateTime<Local>`](chrono::DateTime).
type LocalDateTime = chrono::DateTime<Local>;

/// A wrapper type for [`chrono::DateTime<Local>`](chrono::DateTime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct DateTime(LocalDateTime);

impl DateTime {
    /// Returns a new instance which corresponds to the current date and time.
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

    /// Returns the number of non-leap nanoseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn current_timestamp_nanos() -> i64 {
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap seconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp(secs: i64) -> Self {
        let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
        Self(dt.with_timezone(&Local))
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap milliseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp_millis(millis: i64) -> Self {
        let dt = chrono::DateTime::from_timestamp_millis(millis).unwrap_or_default();
        Self(dt.with_timezone(&Local))
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap microseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp_micros(micros: i64) -> Self {
        let dt = chrono::DateTime::from_timestamp_micros(micros).unwrap_or_default();
        Self(dt.with_timezone(&Local))
    }

    /// Returns a new instance corresponding to a UTC date and time,
    /// from the number of non-leap nanoseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn from_timestamp_nanos(nanos: i64) -> Self {
        let dt = chrono::DateTime::from_timestamp_nanos(nanos);
        Self(dt.with_timezone(&Local))
    }

    /// Returns a new instance corresponding to a UTC date and time from a UUID timestamp.
    #[inline]
    pub fn from_uuid_timestamp(ts: Timestamp) -> Self {
        let (secs, nanos) = ts.to_unix();
        let nanos = secs * 1_000_000_000 + u64::from(nanos);
        Self::from_timestamp_nanos(nanos.try_into().unwrap_or_default())
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

    /// Returns the number of non-leap nanoseconds since the midnight UTC on January 1, 1970.
    #[inline]
    pub fn timestamp_nanos(&self) -> i64 {
        self.0.timestamp_nanos_opt().unwrap_or_default()
    }

    /// Returns a timestamp that can be encoded into a UUID.
    pub fn uuid_timestamp(&self) -> Timestamp {
        let secs = self.timestamp().try_into().unwrap_or_default();
        let nanos = self.0.nanosecond();
        Timestamp::from_unix(NoContext, secs, nanos)
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

    /// Returns a date-time string with the `Local` time zone.
    #[inline]
    pub fn to_local_string(&self) -> String {
        format!("{}", self.0.format("%Y-%m-%d %H:%M:%S %:z"))
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
        let duration = if self > &other {
            self.0 - other.0
        } else {
            other.0 - self.0
        };
        duration.to_std().unwrap_or_default()
    }

    /// Returns the duration of time between `self` and `DateTime::now()`.
    #[inline]
    pub fn span_between_now(&self) -> Duration {
        self.span_between(Self::now())
    }

    /// Returns the duration of time from `self` to `DateTime::now()`.
    #[inline]
    pub fn span_before_now(&self) -> Option<Duration> {
        let current = Self::now();
        if self <= &current {
            (current.0 - self.0).to_std().ok()
        } else {
            None
        }
    }

    /// Returns the duration of time from `DateTime::now()` to `self`.
    #[inline]
    pub fn span_after_now(&self) -> Option<Duration> {
        let current = Self::now();
        if self >= &current {
            (self.0 - current.0).to_std().ok()
        } else {
            None
        }
    }

    /// Retrieves the date component.
    #[inline]
    pub fn date(&self) -> Date {
        self.0.date_naive().into()
    }

    /// Retrieves the time component.
    #[inline]
    pub fn time(&self) -> Time {
        self.0.time().into()
    }

    /// Returns the year number in the calendar date.
    #[inline]
    pub fn year(&self) -> i32 {
        self.0.year()
    }

    /// Returns the quarter number starting from 1.
    ///
    /// The return value ranges from 1 to 4.
    #[inline]
    pub fn quarter(&self) -> u32 {
        self.0.month().div_ceil(3)
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

    /// Returns the millisecond number from 0 to 999.
    #[inline]
    pub fn millisecond(&self) -> u32 {
        self.0.timestamp_subsec_millis() % 1000
    }

    /// Returns the microsecond number from 0 to 999.
    #[inline]
    pub fn microsecond(&self) -> u32 {
        self.0.timestamp_subsec_micros() % 1000
    }

    /// Returns the nanosecond number from 0 to 999.
    #[inline]
    pub fn nanosecond(&self) -> u32 {
        self.0.timestamp_subsec_nanos() % 1_000_000
    }

    /// Returns the ISO week number starting from 1.
    ///
    /// The return value ranges from 1 to 53. (The last week of year differs by years.)
    #[inline]
    pub fn week(&self) -> u32 {
        self.0.iso_week().week()
    }

    /// Returns the day of year starting from 1.
    ///
    /// The return value ranges from 1 to 366. (The last day of year differs by years.)
    #[inline]
    pub fn day_of_year(&self) -> u32 {
        self.0.ordinal()
    }

    /// Returns the day of week starting from 0 (Sunday) to 6 (Saturday).
    #[inline]
    pub fn day_of_week(&self) -> u8 {
        self.iso_day_of_week() % 7
    }

    /// Returns the ISO day of week starting from 1 (Monday) to 7 (Sunday).
    #[inline]
    pub fn iso_day_of_week(&self) -> u8 {
        (self.0.weekday() as u8) + 1
    }

    /// Returns `true` if the current year is a leap year.
    #[inline]
    pub fn is_leap_year(&self) -> bool {
        self.0.date_naive().leap_year()
    }

    /// Returns `true` if the current day is weekend.
    #[inline]
    pub fn is_weekend(&self) -> bool {
        matches!(self.0.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// Returns the number of days in the current year.
    #[inline]
    pub fn days_in_current_year(&self) -> u32 {
        if self.is_leap_year() { 366 } else { 365 }
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
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the end of the current year.
    pub fn end_of_current_year(&self) -> Self {
        let year = self.year();
        let dt = NaiveDate::from_ymd_opt(year, 12, 31)
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the start of the current quarter.
    pub fn start_of_current_quarter(&self) -> Self {
        let year = self.year();
        let month = 3 * self.quarter() - 2;
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the end of the current quarter.
    pub fn end_of_current_quarter(&self) -> Self {
        let year = self.year();
        let month = 3 * self.quarter();
        let day = Date::days_in_month(year, month);
        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the start of the current month.
    pub fn start_of_current_month(&self) -> Self {
        let year = self.year();
        let month = self.month();
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
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
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the start of the current day.
    pub fn start_of_current_day(&self) -> Self {
        let date = self.0.date_naive();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the end of the current day.
    pub fn end_of_current_day(&self) -> Self {
        let date = self.0.date_naive();
        let dt = date
            .and_hms_milli_opt(23, 59, 59, 1_000)
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the start of the year.
    pub fn start_of_year(year: i32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the end of the year.
    pub fn end_of_year(year: i32) -> Self {
        let dt = NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .and_then(|date| date.pred_opt())
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the start of the month.
    pub fn start_of_month(year: i32, month: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the end of the month.
    pub fn end_of_month(year: i32, month: u32) -> Self {
        let dt = NaiveDate::from_ymd_opt(year, month + 1, 1)
            .and_then(|date| date.pred_opt())
            .and_then(|date| date.and_hms_milli_opt(23, 59, 59, 1_000))
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the start of the day.
    pub fn start_of_day(year: i32, month: u32, day: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default();
        let dt = NaiveDateTime::new(date, NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
    }

    /// Returns the end of the month.
    pub fn end_of_day(year: i32, month: u32, day: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default();
        let dt = date
            .and_hms_milli_opt(23, 59, 59, 1_000)
            .unwrap_or_default();
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
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
        self.0.format("%Y-%m-%d %H:%M:%S%.6f %z").fmt(f)
    }
}

impl Serialize for DateTime {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_utc_timestamp())
    }
}

impl From<Date> for DateTime {
    fn from(d: Date) -> Self {
        let dt = NaiveDateTime::new(d.into(), NaiveTime::default());
        let offset = Local.offset_from_utc_datetime(&dt);
        Self(LocalDateTime::from_naive_utc_and_offset(
            dt - offset,
            offset,
        ))
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

#[cfg(feature = "i18n")]
impl<'a> From<DateTime> for fluent::FluentValue<'a> {
    #[inline]
    fn from(dt: DateTime) -> Self {
        fluent::FluentValue::String(dt.to_string().into())
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
        } else if s.contains('+') || s.contains(" -") {
            LocalDateTime::from_str(s).map(Self)
        } else if s.ends_with('Z') {
            let dt = s.parse::<chrono::DateTime<Utc>>()?;
            Ok(dt.with_timezone(&Local).into())
        } else {
            let dt = [s, "Z"].concat().parse::<chrono::DateTime<Utc>>()?;
            Ok(dt.with_timezone(&Local).into())
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

#[cfg(feature = "sqlx")]
impl<DB> sqlx::Type<DB> for DateTime
where
    DB: sqlx::Database,
    LocalDateTime: sqlx::Type<DB>,
{
    #[inline]
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <LocalDateTime as sqlx::Type<DB>>::type_info()
    }
}

#[cfg(feature = "sqlx")]
impl<'r, DB> sqlx::Decode<'r, DB> for DateTime
where
    DB: sqlx::Database,
    LocalDateTime: sqlx::Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as sqlx::Database>::ValueRef<'r>) -> Result<Self, crate::BoxError> {
        <LocalDateTime as sqlx::Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{Date, DateTime};

    #[test]
    fn it_parses_datetime() {
        assert!("2023-12-31".parse::<DateTime>().is_ok());
        assert!("2023-12-31T18:00:00".parse::<DateTime>().is_ok());
        assert!("2023-07-13T02:16:33.449Z".parse::<DateTime>().is_ok());
        assert!(
            "2023-06-10 05:17:23.713071 +0800"
                .parse::<DateTime>()
                .is_ok()
        );

        let datetime = "2023-11-30 16:24:30.654321 +0800"
            .parse::<DateTime>()
            .unwrap();
        let start_day = datetime.start_of_current_day();
        let end_day = datetime.end_of_current_day();
        assert_eq!("2023-11-30", start_day.format_date());
        assert_eq!("00:00:00", start_day.format_time());
        assert_eq!("2023-11-30", end_day.format_date());
        assert_eq!("23:59:60", end_day.format_time());

        let date = "2023-11-30".parse::<Date>().unwrap();
        let datetime = DateTime::from(date);
        assert!(datetime.day_of_week() == 4);
        assert_eq!("2023-11-30", datetime.format_date());
        assert_eq!("00:00:00", datetime.format_time());
    }
}
