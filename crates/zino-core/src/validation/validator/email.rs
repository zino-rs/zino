use self::InvalidEmail::*;
use super::Validator;
use crate::LazyLock;
use regex::Regex;
use std::{fmt, net::IpAddr, str::FromStr};

/// A validator for the email address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmailValidator;

/// An error for the email address validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidEmail {
    /// The value is empty.
    Empty,
    /// The `@` symbol is missing.
    MissingAt,
    /// The user info is too long.
    UserLengthExceeded,
    /// Invalid user.
    InvalidUser,
    /// The domain info is too long.
    DomainLengthExceeded,
    /// Invalid domain.
    InvalidDomain,
}

impl fmt::Display for InvalidEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Empty => write!(f, "value is empty"),
            MissingAt => write!(f, "value is missing `@`"),
            UserLengthExceeded => {
                write!(f, "user length exceeded maximum of 64 characters")
            }
            InvalidUser => write!(f, "user contains unexpected characters"),
            DomainLengthExceeded => {
                write!(f, "domain length exceeded maximum of 255 characters")
            }
            InvalidDomain => write!(f, "domain contains unexpected characters"),
        }
    }
}

impl std::error::Error for InvalidEmail {}

impl Validator<str> for EmailValidator {
    type Error = InvalidEmail;

    fn validate(&self, data: &str) -> Result<(), Self::Error> {
        if data.is_empty() {
            return Err(Empty);
        }

        let (user, domain) = data.split_once('@').ok_or(MissingAt)?;
        if user.len() > 64 {
            return Err(UserLengthExceeded);
        }
        if !EMAIL_USER_PATTERN.is_match(user) {
            return Err(InvalidUser);
        }
        if domain.len() > 255 {
            return Err(DomainLengthExceeded);
        }
        if !EMAIL_DOMAIN_PATTERN.is_match(domain)
            && domain
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .and_then(|s| IpAddr::from_str(s).ok())
                .is_none()
        {
            return Err(InvalidDomain);
        }
        Ok(())
    }
}

/// Regex for the email user.
static EMAIL_USER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i-u)^[a-z0-9.!#$%&'*+/=?^_`{|}~-]+\z")
        .expect("fail to create a regex for the email user")
});

/// Regex for the email domain.
static EMAIL_DOMAIN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i-u)^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$",
    )
    .expect("fail to create a regex for the email domain")
});
