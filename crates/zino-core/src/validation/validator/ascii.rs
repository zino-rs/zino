use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiValidator;

impl Validator<str> for AsciiValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii() {
                bail!(
                    "the char `{}` at the index `{}` is not an ASCII character",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
