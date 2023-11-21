use super::Validator;
use crate::{bail, error::Error};

/// A validator for alphabetic characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlphabeticValidator;

impl Validator<str> for AlphabeticValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_alphabetic() {
                bail!(
                    "the char `{}` at the index `{}` is not alphabetic",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
