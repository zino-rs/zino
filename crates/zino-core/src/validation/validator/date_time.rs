use super::Validator;
use crate::datetime::DateTime;
use chrono::format::ParseError;
use std::str::FromStr;

/// A validator for [`DateTime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeValidator;

impl Validator<str> for DateTimeValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        DateTime::from_str(data)?;
        Ok(())
    }
}
