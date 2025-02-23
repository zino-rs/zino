use super::Validator;
use crate::{bail, error::Error};

/// A validator for alphabetic and numeric characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlphanumericValidator;

impl Validator<str> for AlphanumericValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_alphanumeric() {
                bail!("char `{}` at the index `{}` is not alphanumeric", ch, index);
            }
        }
        Ok(())
    }
}
