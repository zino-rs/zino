use super::Validator;
use crate::{bail, error::Error};

/// A validator for numeric characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericValidator;

impl Validator<str> for NumericValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        for (index, ch) in data.char_indices() {
            if !ch.is_numeric() {
                bail!("the char `{}` at the index `{}` is not numeric", ch, index);
            }
        }
        Ok(())
    }
}
