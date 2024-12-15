use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII alphabetic characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiAlphabeticValidator;

impl Validator<str> for AsciiAlphabeticValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii_alphabetic() {
                bail!(
                    "the char `{}` at the index `{}` is not ASCII alphabetic",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
