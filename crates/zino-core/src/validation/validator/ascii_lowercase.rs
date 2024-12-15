use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII lowercase characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiLowercaseValidator;

impl Validator<str> for AsciiLowercaseValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii_lowercase() {
                bail!(
                    "the char `{}` at the index `{}` is not ASCII lowercase",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
