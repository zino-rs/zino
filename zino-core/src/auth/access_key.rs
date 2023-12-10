use crate::{
    crypto::{self, Digest},
    encoding::base64,
    extension::TomlTableExt,
    state::State,
};
use hmac::{
    digest::{FixedOutput, KeyInit, MacMarker, Update},
    Hmac, Mac,
};
use rand::{distributions::Alphanumeric, Rng};
use std::{borrow::Cow, fmt, iter, sync::LazyLock};

#[cfg(feature = "auth-totp")]
use totp_rs::{Algorithm, TOTP};

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
pub struct SecretAccessKey(Vec<u8>);

impl SecretAccessKey {
    /// Creates a new instance for the Access key ID.
    #[inline]
    pub fn new(access_key_id: &AccessKeyId) -> Self {
        Self::with_key::<Hmac<Digest>>(access_key_id, SECRET_KEY.as_ref())
    }

    /// Creates a new instance with the specific key.
    pub fn with_key<H>(access_key_id: &AccessKeyId, key: impl AsRef<[u8]>) -> Self
    where
        H: FixedOutput + KeyInit + MacMarker + Update,
    {
        let mut mac = H::new_from_slice(key.as_ref()).expect("HMAC can take key of any size");
        mac.update(access_key_id.as_ref());
        Self(mac.finalize().into_bytes().to_vec())
    }

    /// Returns a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Consumes `self` and generates a TOTP used for 2FA authentification.
    #[cfg(feature = "auth-totp")]
    pub fn generate_totp(self, issuer: Option<String>, account_name: String) -> TOTP {
        let mut secret = self.0;
        secret.truncate(20);
        TOTP::new_unchecked(Algorithm::SHA1, 6, 1, 30, secret, issuer, account_name)
    }
}

impl fmt::Display for SecretAccessKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode(self.as_bytes()))
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
    let app_config = State::shared().config();
    let config = app_config.get_table("access-key").unwrap_or(app_config);
    let checksum: [u8; 32] = config
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().first_chunk().copied())
        .unwrap_or_else(|| {
            let secret = config.get_str("secret").unwrap_or_else(|| {
                tracing::warn!("an auto-generated `secret` is used for deriving a secret key");
                crate::application::APP_NMAE.as_ref()
            });
            crypto::digest(secret.as_bytes())
        });
    let info = config.get_str("info").unwrap_or("ZINO:ACCESS-KEY");
    crypto::derive_key(info, &checksum)
});
