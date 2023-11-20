use super::Validator;
use std::{
    net::{AddrParseError, IpAddr},
    str::FromStr,
};

/// A validator for [`IpAddr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpAddrValidator;

impl Validator<str> for IpAddrValidator {
    type Error = AddrParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        IpAddr::from_str(data)?;
        Ok(())
    }
}
