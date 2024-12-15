use super::Validator;
use std::str::FromStr;
use uuid::{Error, Uuid};

/// A validator for [`Uuid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UuidValidator;

impl Validator<str> for UuidValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Uuid::from_str(data)?;
        Ok(())
    }
}
