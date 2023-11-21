use super::Validator;
use crate::{bail, error::Error};

/// A validator for ASCII uppercase characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiUppercaseValidator;

impl Validator<str> for AsciiUppercaseValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_ascii_uppercase() {
                bail!(
                    "the char `{}` at the index `{}` is not ASCII uppercase",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
