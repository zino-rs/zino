use super::Validator;
use crate::datetime::Date;
use chrono::format::ParseError;
use std::str::FromStr;

/// A validator for [`Date`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateValidator;

impl Validator<str> for DateValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Date::from_str(data)?;
        Ok(())
    }
}
