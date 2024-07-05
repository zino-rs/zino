use self::ParseSecurityTokenError::*;
use super::AccessKeyId;
use crate::{crypto, datetime::DateTime, encoding::base64, error::Error, warn};
use std::{fmt, time::Duration};

/// Security token.
#[derive(Debug, Clone)]
pub struct SecurityToken {
    /// Access key ID.
    access_key_id: AccessKeyId,
    /// Expires time.
    expires_at: DateTime,
    /// Token.
    token: String,
}

impl SecurityToken {
    /// Attempts to create a new instance.
    pub fn try_new(
        access_key_id: AccessKeyId,
        expires_at: DateTime,
        key: impl AsRef<[u8]>,
    ) -> Result<Self, Error> {
        fn inner(
            access_key_id: AccessKeyId,
            expires_at: DateTime,
            key: &[u8],
        ) -> Result<SecurityToken, Error> {
            let signature = format!("{}:{}", &access_key_id, expires_at.timestamp());
            let authorization = crypto::encrypt(signature.as_bytes(), key)?;
            let token = base64::encode(authorization);
            Ok(SecurityToken {
                access_key_id,
                expires_at,
                token,
            })
        }
        inner(access_key_id, expires_at, key.as_ref())
    }

    /// Returns the access key ID.
    #[inline]
    pub fn access_key_id(&self) -> &AccessKeyId {
        &self.access_key_id
    }

    /// Returns the expires time.
    #[inline]
    pub fn expires_at(&self) -> DateTime {
        self.expires_at
    }

    /// Returns the time when the security token will expire in.
    #[inline]
    pub fn expires_in(&self) -> Duration {
        self.expires_at.span_after_now().unwrap_or_default()
    }

    /// Returns `true` if the security token has expired.
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.expires_at <= DateTime::now()
    }

    /// Returns a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.token.as_str()
    }

    /// Parses the token with the encryption key.
    pub(crate) fn parse_with(token: String, key: &[u8]) -> Result<Self, ParseSecurityTokenError> {
        let authorization = base64::decode(&token).map_err(|err| DecodeError(err.into()))?;
        let signature = crypto::decrypt(&authorization, key)
            .map_err(|_| DecodeError(warn!("fail to decrypt authorization")))?;
        let signature_str = String::from_utf8_lossy(&signature);
        if let Some((access_key_id, timestamp)) = signature_str.split_once(':') {
            let timestamp = timestamp
                .parse::<i64>()
                .map_err(|err| ParseExpiresError(err.into()))?;
            let expires_at = DateTime::from_timestamp(timestamp);
            if expires_at >= DateTime::now() {
                Ok(Self {
                    access_key_id: access_key_id.into(),
                    expires_at,
                    token,
                })
            } else {
                Err(ValidPeriodExpired(expires_at))
            }
        } else {
            Err(InvalidFormat)
        }
    }
}

impl fmt::Display for SecurityToken {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl AsRef<[u8]> for SecurityToken {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.token.as_ref()
    }
}

/// An error which can be returned when parsing a token.
#[derive(Debug)]
pub(crate) enum ParseSecurityTokenError {
    /// An error that can occur while decoding.
    DecodeError(Error),
    /// An error which can occur while parsing a expires timestamp.
    ParseExpiresError(Error),
    /// Valid period expired.
    ValidPeriodExpired(DateTime),
    /// Invalid format.
    InvalidFormat,
}

impl fmt::Display for ParseSecurityTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError(err) => write!(f, "decode error: {err}"),
            ParseExpiresError(err) => write!(f, "parse expires error: {err}"),
            ValidPeriodExpired(expires) => write!(f, "expired at `{expires}`"),
            InvalidFormat => write!(f, "invalid format"),
        }
    }
}

impl std::error::Error for ParseSecurityTokenError {}
