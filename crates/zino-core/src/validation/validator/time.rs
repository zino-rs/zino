use super::Validator;
use crate::datetime::Time;
use chrono::format::ParseError;
use std::str::FromStr;

/// A validator for [`Time`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeValidator;

impl Validator<str> for TimeValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Time::from_str(data)?;
        Ok(())
    }
}
