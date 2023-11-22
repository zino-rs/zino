use super::Validator;
use crate::{bail, error::Error};

/// A validator for a credit card number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreditCardValidator;

impl Validator<str> for CreditCardValidator {
    type Error = Error;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        if card_validate::Validate::from(data).is_err() {
            bail!("invalid credit card number");
        }
        Ok(())
    }
}
