use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII hexadecimal digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiHexdigitValidator;

impl Validator<str> for AsciiHexdigitValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii_hexdigit() {
                bail!(
                    "the char `{}` at the index `{}` is not an ASCII hexadecimal digit",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
