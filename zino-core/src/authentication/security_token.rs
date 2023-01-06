use crate::{crypto, AccessKeyId, DateTime};
use std::fmt;

/// An error which can be returned when parsing a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseTokenError {
    /// An error that can occur while decoding.
    DecodeError(String),
    /// Invalid format.
    InvalidFormat,
    /// An error which can occur while parsing a expires timestamp.
    ParseExpiresError(String),
    /// Valid period expired.
    ValidPeriodExpired,
}

impl fmt::Display for ParseTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseTokenError::*;
        match self {
            DecodeError(s) => {
                write!(f, "decode error: {s}")
            }
            InvalidFormat => write!(f, "invalid format"),
            ParseExpiresError(s) => {
                write!(f, "parse expires error: {s}")
            }
            ValidPeriodExpired => write!(f, "valid period has expired"),
        }
    }
}

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
    pub fn new(key: impl AsRef<[u8]>, id: impl Into<AccessKeyId>, expires: DateTime) -> Self {
        let key = key.as_ref();
        let grantor_id = id.into();
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

    /// Parses the token with the encryption key.
    pub(crate) fn parse_token(key: &[u8], token: String) -> Result<Self, ParseTokenError> {
        use ParseTokenError::*;
        match base64::decode(&token) {
            Ok(data) => {
                let authorization = crypto::decrypt(key, &data)
                    .map_err(|_| DecodeError("fail to decrypt authorization".to_string()))?;
                if let Some((assignee_id, timestamp)) = authorization.split_once(':') {
                    match timestamp.parse() {
                        Ok(secs) => {
                            if DateTime::now().timestamp() <= secs {
                                let expires = DateTime::from_timestamp(secs);
                                let grantor_id = crypto::decrypt(key, assignee_id.as_ref())
                                    .map_err(|_| {
                                        DecodeError("fail to decrypt grantor id".to_string())
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
                        Err(err) => Err(ParseExpiresError(err.to_string())),
                    }
                } else {
                    Err(InvalidFormat)
                }
            }
            Err(err) => Err(DecodeError(err.to_string())),
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
            .ok()
            .map(base64::encode)
    }

    /// Decrypts the data using AES-GCM-SIV.
    pub fn decrypt(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Option<String> {
        base64::decode(data)
            .ok()
            .and_then(|cipher| crypto::decrypt(key.as_ref(), &cipher).ok())
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
