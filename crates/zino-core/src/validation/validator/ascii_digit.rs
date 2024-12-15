use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII decimal digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiDigitValidator;

impl Validator<str> for AsciiDigitValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii_digit() {
                bail!(
                    "the char `{}` at the index `{}` is not an ASCII decimal digit",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
