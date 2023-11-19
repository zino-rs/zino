use super::Validator;
use chrono::{format::ParseError, NaiveDate};
use std::str::FromStr;

/// A validator for date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateValidator;

impl Validator<str> for DateValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        NaiveDate::from_str(data)?;
        Ok(())
    }
}
