use super::Validator;
use regex::{Error, Regex};
use std::str::FromStr;

/// A validator for regular expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegexValidator;

impl Validator<str> for RegexValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Regex::from_str(data)?;
        Ok(())
    }
}
