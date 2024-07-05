use crate::{encoding::base64, error::Error, validation::Validation, SharedString};
use hmac::digest::{Digest, FixedOutput, HashMarker, Update};
use serde::{Deserialize, Serialize};
use std::{error, fmt};

/// Session Identification URI.
/// See [the spec](https://www.w3.org/TR/WD-session-id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionId {
    /// Specifies the realm within which linkage of the identifier is possible.
    /// Realms have the same format as DNS names.
    realm: SharedString,
    /// Unstructured random integer specific to realm generated using a procedure with
    /// a negligible probability of collision. The identifier is encoded using base64.
    identifier: String,
    /// Optional extension of identifier field used to differentiate concurrent uses of
    /// the same session identifier. The thread field is an integer encoded in hexadecimal.
    thread: u8,
    /// Optional Hexadecimal encoded integer containing a monotonically increasing counter value.
    /// A client should increment the count field after each operation.
    count: u8,
}

impl SessionId {
    /// Creates a new instance.
    #[inline]
    pub fn new<D>(realm: impl Into<SharedString>, key: impl AsRef<[u8]>) -> Self
    where
        D: Default + FixedOutput + HashMarker + Update,
    {
        fn inner<D>(realm: SharedString, key: &[u8]) -> SessionId
        where
            D: Default + FixedOutput + HashMarker + Update,
        {
            let data = [realm.as_ref().as_bytes(), key].concat();
            let mut hasher = D::new();
            hasher.update(data.as_ref());

            let identifier = base64::encode(hasher.finalize().as_slice());
            SessionId {
                realm,
                identifier,
                thread: 0,
                count: 0,
            }
        }
        inner::<D>(realm.into(), key.as_ref())
    }

    /// Validates the session identifier using the realm and the key.
    pub fn validate_with<D>(&self, realm: &str, key: impl AsRef<[u8]>) -> Validation
    where
        D: Default + FixedOutput + HashMarker + Update,
    {
        fn inner<D>(session_id: &SessionId, realm: &str, key: &[u8]) -> Validation
        where
            D: Default + FixedOutput + HashMarker + Update,
        {
            let mut validation = Validation::new();
            let identifier = &session_id.identifier;
            match base64::decode(identifier) {
                Ok(hash) => {
                    let data = [realm.as_bytes(), key].concat();
                    let mut hasher = D::new();
                    hasher.update(data.as_ref());

                    if hasher.finalize().as_slice() != hash {
                        validation.record("identifier", "invalid session identifier");
                    }
                }
                Err(err) => {
                    validation.record_fail("identifier", err);
                }
            }
            validation
        }
        inner::<D>(self, realm, key.as_ref())
    }

    /// Returns `true` if the given `SessionId` can be accepted by `self`.
    pub fn accepts(&self, session_id: &SessionId) -> bool {
        if self.identifier() != session_id.identifier() {
            return false;
        }

        let realm = self.realm();
        let domain = session_id.realm();
        if domain == realm {
            self.count() <= session_id.count()
        } else {
            let remainder = if realm.len() > domain.len() {
                realm.strip_suffix(domain)
            } else {
                domain.strip_suffix(realm)
            };
            remainder.is_some_and(|s| s.ends_with('.'))
        }
    }

    /// Sets the thread used to differentiate concurrent uses of the same session identifier.
    #[inline]
    pub fn set_thread(&mut self, thread: u8) {
        self.thread = thread;
    }

    /// Increments the count used to prevent replay attacks.
    #[inline]
    pub fn increment_count(&mut self) {
        self.count = self.count.saturating_add(1);
    }

    /// Returns the realm as `&str`.
    #[inline]
    pub fn realm(&self) -> &str {
        self.realm.as_ref()
    }

    /// Returns the identifier as `&str`.
    #[inline]
    pub fn identifier(&self) -> &str {
        self.identifier.as_ref()
    }

    /// Returns the thread.
    #[inline]
    pub fn thread(&self) -> u8 {
        self.thread
    }

    /// Returns the count.
    #[inline]
    pub fn count(&self) -> u8 {
        self.count
    }

    /// Parses the `SessionId`.
    pub(crate) fn parse(s: &str) -> Result<SessionId, ParseSessionIdError> {
        use ParseSessionIdError::*;
        if let Some(s) = s.strip_prefix("SID:ANON:") {
            if let Some((realm, s)) = s.split_once(':') {
                if let Some((identifier, s)) = s.split_once('-') {
                    if let Some((thread, count)) = s.split_once(':') {
                        return u8::from_str_radix(thread, 16)
                            .map_err(|err| ParseThreadError(err.into()))
                            .and_then(|thread| {
                                u8::from_str_radix(count, 16)
                                    .map_err(|err| ParseCountError(err.into()))
                                    .map(|count| Self {
                                        realm: realm.to_owned().into(),
                                        identifier: identifier.to_owned(),
                                        thread,
                                        count,
                                    })
                            });
                    } else {
                        return u8::from_str_radix(s, 16)
                            .map_err(|err| ParseThreadError(err.into()))
                            .map(|thread| Self {
                                realm: realm.to_owned().into(),
                                identifier: identifier.to_owned(),
                                thread,
                                count: 0,
                            });
                    }
                } else if let Some((identifier, count)) = s.split_once(':') {
                    return u8::from_str_radix(count, 16)
                        .map_err(|err| ParseCountError(err.into()))
                        .map(|count| Self {
                            realm: realm.to_owned().into(),
                            identifier: identifier.to_owned(),
                            thread: 0,
                            count,
                        });
                } else {
                    return Ok(Self {
                        realm: realm.to_owned().into(),
                        identifier: s.to_owned(),
                        thread: 0,
                        count: 0,
                    });
                }
            }
        }
        Err(InvalidFormat)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let realm = &self.realm;
        let identifier = &self.identifier;
        let thread = self.thread;
        let count = self.count;
        if thread > 0 {
            if count > 0 {
                write!(f, "SID:ANON:{realm}:{identifier}-{thread:x}:{count:x}")
            } else {
                write!(f, "SID:ANON:{realm}:{identifier}-{thread:x}")
            }
        } else if count > 0 {
            write!(f, "SID:ANON:{realm}:{identifier}:{count:x}")
        } else {
            write!(f, "SID:ANON:{realm}:{identifier}")
        }
    }
}

/// An error which can be returned when parsing a `SessionId`.
#[derive(Debug)]
pub(crate) enum ParseSessionIdError {
    /// An error that can occur when parsing thread.
    ParseThreadError(Error),
    /// An error that can occur when parsing count.
    ParseCountError(Error),
    /// Invalid format.
    InvalidFormat,
}

impl fmt::Display for ParseSessionIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseSessionIdError::*;
        match self {
            ParseThreadError(err) => write!(f, "fail to parse thread: {err}"),
            ParseCountError(err) => write!(f, "fail to parse count: {err}"),
            InvalidFormat => write!(f, "invalid format"),
        }
    }
}

impl error::Error for ParseSessionIdError {}
