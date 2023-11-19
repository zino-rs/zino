use super::Validator;
use std::{
    net::{AddrParseError, Ipv6Addr},
    str::FromStr,
};

/// A validator for [`Ipv6Addr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv6AddrValidator;

impl Validator<str> for Ipv6AddrValidator {
    type Error = AddrParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Ipv6Addr::from_str(data)?;
        Ok(())
    }
}
