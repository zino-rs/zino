use crate::{error::Error, AvroValue, JsonValue};
use chrono::{format::ParseError, Datelike, Days, Local, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
    time::Duration,
};

/// A wrapper type for [`chrono::NaiveDate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct Date(NaiveDate);

impl Date {
    /// Attempts to create a new instance.
    #[inline]
    pub fn try_new(year: i32, month: u32, day: u32) -> Result<Self, Error> {
        NaiveDate::from_ymd_opt(year, month, day)
            .map(Self)
            .ok_or_else(|| {
                let message = format!(
                    "fail to create a date from year: `{year}`, month: `{month}`, day: `{day}`"
                );
                Error::new(message)
            })
    }

    /// Returns a new instance which corresponds to the current date.
    #[inline]
    pub fn today() -> Self {
        Self(Local::now().date_naive())
    }

    /// Returns a new instance which corresponds to the tomorrow date.
    #[inline]
    pub fn tomorrow() -> Self {
        let date = Local::now()
            .date_naive()
            .succ_opt()
            .unwrap_or(NaiveDate::MAX);
        Self(date)
    }

    /// Returns a new instance which corresponds to the yesterday date.
    #[inline]
    pub fn yesterday() -> Self {
        let date = Local::now()
            .date_naive()
            .pred_opt()
            .unwrap_or(NaiveDate::MIN);
        Self(date)
    }

    /// Returns a new instance which corresponds to 1st of January 1970.
    #[inline]
    pub fn epoch() -> Self {
        Self(NaiveDate::default())
    }

    /// Counts the days from the 1st of January 1970.
    #[inline]
    pub fn num_days_from_epoch(&self) -> i32 {
        let unix_epoch = NaiveDate::default();
        self.0
            .signed_duration_since(unix_epoch)
            .num_days()
            .try_into()
            .unwrap_or_default()
    }

    /// Formats the date with the specified format string.
    /// See [`format::strftime`](chrono::format::strftime) for the supported escape sequences.
    #[inline]
    pub fn format(&self, fmt: &str) -> String {
        format!("{}", self.0.format(fmt))
    }

    /// Returns the amount of time elapsed from another date to this one,
    /// or zero duration if that date is later than this one.
    #[inline]
    pub fn duration_since(&self, earlier: Date) -> Duration {
        (self.0 - earlier.0).to_std().unwrap_or_default()
    }

    /// Returns the duration of time between `self` and `other`.
    #[inline]
    pub fn span_between(&self, other: Date) -> Duration {
        let duration = if self > &other {
            self.0 - other.0
        } else {
            other.0 - self.0
        };
        duration.to_std().unwrap_or_default()
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
        self.0.leap_year()
    }

    /// Returns `true` if the current day is weekend.
    #[inline]
    pub fn is_weekend(&self) -> bool {
        matches!(self.0.weekday(), Weekday::Sat | Weekday::Sun)
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
        Self(NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default())
    }

    /// Returns the end of the current year.
    pub fn end_of_current_year(&self) -> Self {
        let year = self.year();
        Self(NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or_default())
    }

    /// Returns the start of the next year.
    pub fn start_of_next_year(&self) -> Self {
        let year = self.year() + 1;
        Self(NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default())
    }

    /// Returns the start of the current month.
    pub fn start_of_current_month(&self) -> Self {
        let year = self.year();
        let month = self.month();
        Self(NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default())
    }

    /// Returns the end of the current month.
    pub fn end_of_current_month(&self) -> Self {
        let year = self.year();
        let month = self.month();
        let day = self.days_in_current_month();
        Self(NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default())
    }

    /// Returns the start of the next month.
    pub fn start_of_next_month(&self) -> Self {
        let year = self.year();
        let month = self.month();
        let date_opt = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month, 1)
        };
        Self(date_opt.unwrap_or_default())
    }

    /// Adds a duration in months to the date.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_add_months(self, months: u32) -> Option<Self> {
        self.0.checked_add_months(Months::new(months)).map(Self)
    }

    /// Subtracts a duration in months from the date.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_sub_months(self, months: u32) -> Option<Self> {
        self.0.checked_sub_months(Months::new(months)).map(Self)
    }

    /// Adds a duration in days to the date.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_add_days(self, days: u32) -> Option<Self> {
        self.0
            .checked_add_days(Days::new(u64::from(days)))
            .map(Self)
    }

    /// Subtracts a duration in days from the date.
    /// Returns `None` if the resulting date would be out of range.
    #[inline]
    pub fn checked_sub_days(self, days: u32) -> Option<Self> {
        self.0
            .checked_sub_days(Days::new(u64::from(days)))
            .map(Self)
    }
}

impl Default for Date {
    /// Returns an instance which corresponds to **the current date**.
    #[inline]
    fn default() -> Self {
        Self::today()
    }
}

impl fmt::Display for Date {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl Serialize for Date {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<NaiveDate> for Date {
    #[inline]
    fn from(d: NaiveDate) -> Self {
        Self(d)
    }
}

impl From<Date> for NaiveDate {
    #[inline]
    fn from(d: Date) -> Self {
        d.0
    }
}

impl From<Date> for AvroValue {
    #[inline]
    fn from(d: Date) -> Self {
        AvroValue::Date(d.num_days_from_epoch())
    }
}

impl From<Date> for JsonValue {
    #[inline]
    fn from(d: Date) -> Self {
        JsonValue::String(d.to_string())
    }
}

impl FromStr for Date {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<NaiveDate>().map(Self)
    }
}

impl Add<Duration> for Date {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self {
        let duration = chrono::Duration::from_std(rhs).expect("Duration value is out of range");
        let date = self
            .0
            .checked_add_signed(duration)
            .expect("`Date + Duration` overflowed");
        Self(date)
    }
}

impl AddAssign<Duration> for Date {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for Date {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Duration) -> Self {
        let duration = chrono::Duration::from_std(rhs).expect("Duration value is out of range");
        let date = self
            .0
            .checked_sub_signed(duration)
            .expect("`Date - Duration` overflowed");
        Self(date)
    }
}

impl SubAssign<Duration> for Date {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
