use super::Validator;
use http::uri::{InvalidUri, Uri};
use std::str::FromStr;

/// A validator for [`Uri`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UriValidator;

impl Validator<str> for UriValidator {
    type Error = InvalidUri;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Uri::from_str(data)?;
        Ok(())
    }
}
