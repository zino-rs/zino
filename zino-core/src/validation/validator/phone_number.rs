use super::Validator;
use phonenumber::{ParseError, PhoneNumber};
use std::str::FromStr;

/// A validator for a phone number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhoneNumberValidator;

impl Validator<str> for PhoneNumberValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        PhoneNumber::from_str(data)?;
        Ok(())
    }
}
