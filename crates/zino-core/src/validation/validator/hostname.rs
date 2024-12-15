use super::Validator;
use url::{Host, ParseError};

/// A validator for the hostname of a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostnameValidator;

impl Validator<str> for HostnameValidator {
    type Error = ParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Host::parse(data)?;
        Ok(())
    }
}
