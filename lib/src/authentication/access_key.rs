use hmac::{Hmac, Mac};
use rand::{distributions::Alphanumeric, Rng};
use sha2::Sha256;
use std::{fmt, iter};

/// Access key ID.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessKeyId(String);

impl AccessKeyId {
    /// Creates a new instance with random alphanumeric characters.
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let chars: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(20)
            .collect();
        Self(chars)
    }

    /// Returns a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for AccessKeyId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<[u8]> for AccessKeyId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<String> for AccessKeyId {
    #[inline]
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AccessKeyId {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Secrect access key.
#[derive(Debug, Clone)]
pub struct SecretAccessKey(String);

impl SecretAccessKey {
    /// Creates a new instance.
    pub fn new(key: impl AsRef<[u8]>, id: impl Into<AccessKeyId>) -> Self {
        let data = id.into();
        let mut mac =
            Hmac::<Sha256>::new_from_slice(key.as_ref()).expect("HMAC can take key of any size");
        mac.update(data.as_ref());
        Self(base64::encode(mac.finalize().into_bytes()))
    }

    /// Returns a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for SecretAccessKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<[u8]> for SecretAccessKey {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
