use super::Validator;
use crate::{bail, error::Error};

/// A validator for uppercase characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UppercaseValidator;

impl Validator<str> for UppercaseValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_uppercase() {
                bail!(
                    "the char `{}` at the index `{}` is not uppercase",
                    ch,
                    index
                );
            }
        }
        Ok(())
    }
}
