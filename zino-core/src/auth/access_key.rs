use crate::{crypto, encoding::base64, extension::TomlTableExt, state::State};
use hmac::{
    digest::{FixedOutput, KeyInit, MacMarker, Update},
    Hmac, Mac,
};
use rand::{distributions::Alphanumeric, Rng};
use sha2::Sha256;
use std::{borrow::Cow, env, fmt, iter, sync::LazyLock};

/// Access key ID.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessKeyId(String);

impl AccessKeyId {
    /// Creates a new instance with random alphanumeric characters.
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let chars: String = iter::repeat(())
            .map(|_| rng.sample(Alphanumeric))
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
        Self(s.to_owned())
    }
}

impl<'a> From<Cow<'a, str>> for AccessKeyId {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        Self(s.into_owned())
    }
}

/// Secrect access key.
#[derive(Debug, Clone)]
pub struct SecretAccessKey(String);

impl SecretAccessKey {
    /// Creates a new instance for the Access key ID.
    #[inline]
    pub fn new(access_key_id: &AccessKeyId) -> Self {
        Self::with_key::<Hmac<Sha256>>(access_key_id, SECRET_KEY.as_ref())
    }

    /// Creates a new instance with the specific key.
    pub fn with_key<H>(access_key_id: &AccessKeyId, key: impl AsRef<[u8]>) -> Self
    where
        H: FixedOutput + KeyInit + MacMarker + Update,
    {
        let mut mac = H::new_from_slice(key.as_ref()).expect("HMAC can take key of any size");
        mac.update(access_key_id.as_ref());
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

/// Shared secret.
static SECRET_KEY: LazyLock<[u8; 64]> = LazyLock::new(|| {
    let config = State::shared().config();
    let checksum: [u8; 32] = config
        .get_table("access-key")
        .and_then(|t| t.get_str("checksum"))
        .and_then(|checksum| checksum.as_bytes().first_chunk().copied())
        .unwrap_or_else(|| {
            tracing::warn!("the `checksum` is not set properly for deriving a secret key");

            let app_name = config
                .get_str("name")
                .map(|s| s.to_owned())
                .unwrap_or_else(|| {
                    env::var("CARGO_PKG_NAME")
                        .expect("fail to get the environment variable `CARGO_PKG_NAME`")
                });
            crypto::digest(app_name.as_bytes())
        });
    crypto::derive_key("ZINO:ACCESS-KEY", &checksum)
});
