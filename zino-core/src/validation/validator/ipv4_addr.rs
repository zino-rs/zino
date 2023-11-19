use super::Validator;
use std::{
    net::{AddrParseError, Ipv4Addr},
    str::FromStr,
};

/// A validator for [`Ipv4Addr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4AddrValidator;

impl Validator<str> for Ipv4AddrValidator {
    type Error = AddrParseError;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        Ipv4Addr::from_str(data)?;
        Ok(())
    }
}
