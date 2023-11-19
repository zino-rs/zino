use super::Validator;
use chrono::{format::ParseError, NaiveTime};
use std::str::FromStr;

/// A validator for time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeValidator;

impl Validator<str> for TimeValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        NaiveTime::from_str(data)?;
        Ok(())
    }
}
