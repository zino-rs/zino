use self::InvalidHost::*;
use super::Validator;
use std::{fmt, num::ParseIntError};
use url::{Host, ParseError};

/// A validator for the host of a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostValidator;

/// An error for the host validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidHost {
    /// Invalid port.
    InvalidPort(ParseIntError),
    /// Invalid hostname.
    InvalidHostname(ParseError),
}

impl fmt::Display for InvalidHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidPort(err) => write!(f, "invalid port: {err}"),
            InvalidHostname(err) => write!(f, "invalid hostname: {err}"),
        }
    }
}

impl std::error::Error for InvalidHost {}

impl Validator<str> for HostValidator {
    type Error = InvalidHost;

    #[inline]
    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        if let Some((hostname, port)) = data.rsplit_once(':') {
            if let Err(err) = port.parse::<u16>() {
                Err(InvalidPort(err))
            } else if let Err(err) = Host::parse(hostname) {
                Err(InvalidHostname(err))
            } else {
                Ok(())
            }
        } else if let Err(err) = Host::parse(data) {
            Err(InvalidHostname(err))
        } else {
            Ok(())
        }
    }
}
