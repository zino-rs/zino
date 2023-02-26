use self::ParseSecurityTokenError::*;
use super::AccessKeyId;
use crate::{crypto, datetime::DateTime, format::base64, BoxError};
use std::{error::Error, fmt};

/// Security token.
#[derive(Debug, Clone)]
pub struct SecurityToken {
    /// Grantor ID.
    grantor_id: AccessKeyId,
    /// Assignee ID.
    assignee_id: AccessKeyId,
    /// Expires.
    expires: DateTime,
    /// Token.
    token: String,
}

impl SecurityToken {
    /// Creates a new instance.
    pub fn new(
        key: impl AsRef<[u8]>,
        access_key_id: impl Into<AccessKeyId>,
        expires: DateTime,
    ) -> Self {
        let key = key.as_ref();
        let grantor_id = access_key_id.into();
        let timestamp = expires.timestamp();
        let grantor_id_cipher = crypto::encrypt(key, grantor_id.as_ref()).unwrap_or_default();
        let assignee_id = base64::encode(grantor_id_cipher).into();
        let authorization = format!("{assignee_id}:{timestamp}");
        let authorization_cipher = crypto::encrypt(key, authorization.as_ref()).unwrap_or_default();
        let token = base64::encode(authorization_cipher);
        Self {
            grantor_id,
            assignee_id,
            expires,
            token,
        }
    }

    /// Returns the expires.
    #[inline]
    pub fn expires(&self) -> DateTime {
        self.expires
    }

    /// Returns a reference to the grantor's access key ID.
    #[inline]
    pub fn grantor_id(&self) -> &AccessKeyId {
        &self.grantor_id
    }

    /// Returns a reference to the assignee's access key ID.
    #[inline]
    pub fn assignee_id(&self) -> &AccessKeyId {
        &self.assignee_id
    }

    /// Returns a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.token.as_str()
    }

    /// Encrypts the plaintext using AES-GCM-SIV.
    pub fn encrypt(key: impl AsRef<[u8]>, plaintext: impl AsRef<[u8]>) -> Option<String> {
        crypto::encrypt(key.as_ref(), plaintext.as_ref())
            .inspect_err(|_| tracing::error!("failed to encrypt the plaintext"))
            .ok()
            .map(base64::encode)
    }

    /// Decrypts the data using AES-GCM-SIV.
    pub fn decrypt(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Option<String> {
        base64::decode(data)
            .inspect_err(|_| tracing::error!("failed to encode the data with base64"))
            .ok()
            .and_then(|cipher| {
                crypto::decrypt(key.as_ref(), &cipher)
                    .inspect_err(|_| tracing::error!("failed to decrypt the data"))
                    .ok()
            })
    }

    /// Parses the token with the encryption key.
    pub(crate) fn parse_with(token: String, key: &[u8]) -> Result<Self, ParseSecurityTokenError> {
        match base64::decode(&token) {
            Ok(data) => {
                let authorization = crypto::decrypt(key, &data)
                    .map_err(|_| DecodeError("failed to decrypt authorization".into()))?;
                if let Some((assignee_id, timestamp)) = authorization.split_once(':') {
                    match timestamp.parse() {
                        Ok(secs) => {
                            if DateTime::now().timestamp() <= secs {
                                let expires = DateTime::from_timestamp(secs);
                                let grantor_id = crypto::decrypt(key, assignee_id.as_ref())
                                    .map_err(|_| {
                                        DecodeError("failed to decrypt grantor id".into())
                                    })?;
                                Ok(Self {
                                    grantor_id: grantor_id.into(),
                                    assignee_id: assignee_id.into(),
                                    expires,
                                    token,
                                })
                            } else {
                                Err(ValidPeriodExpired)
                            }
                        }
                        Err(err) => Err(ParseExpiresError(Box::new(err))),
                    }
                } else {
                    Err(InvalidFormat)
                }
            }
            Err(err) => Err(DecodeError(Box::new(err))),
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
    DecodeError(BoxError),
    /// An error which can occur while parsing a expires timestamp.
    ParseExpiresError(BoxError),
    /// Valid period expired.
    ValidPeriodExpired,
    /// Invalid format.
    InvalidFormat,
}

impl fmt::Display for ParseSecurityTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError(err) => write!(f, "decode error: {err}"),
            ParseExpiresError(err) => write!(f, "parse expires error: {err}"),
            ValidPeriodExpired => write!(f, "valid period has expired"),
            InvalidFormat => write!(f, "invalid format"),
        }
    }
}

impl Error for ParseSecurityTokenError {}
