use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII alphabetic and numeric characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiAlphanumericValidator;

impl Validator<str> for AsciiAlphanumericValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii_alphanumeric() {
                bail!(
                    "the char `{}` at the index `{}` is not ASCII alphanumeric",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
