use crate::{
    crypto,
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    state::State,
    JsonValue, Map,
};
use jwt_simple::{
    algorithms::MACLike,
    claims::{self, Claims, JWTClaims},
    common::VerificationOptions,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{env, sync::LazyLock, time::Duration};

/// JWT Claims.
#[derive(Debug, Clone)]
pub struct JwtClaims<T = Map>(pub(crate) JWTClaims<T>);

impl<T: Default + Serialize + DeserializeOwned> JwtClaims<T> {
    /// Creates a new instance.
    pub fn new(subject: impl ToString) -> Self {
        let mut claims = Claims::with_custom_claims(T::default(), (*DEFAULT_MAX_AGE).into());
        claims.invalid_before = None;
        claims.subject = Some(subject.to_string());
        Self(claims)
    }

    /// Creates a new instance, expiring in `max-age`.
    pub fn with_max_age(subject: impl ToString, max_age: Duration) -> Self {
        let mut claims = Claims::with_custom_claims(T::default(), max_age.into());
        claims.invalid_before = None;
        claims.subject = Some(subject.to_string());
        Self(claims)
    }

    /// Generates an access token signed with the shared secret access key.
    pub fn refresh_token(&self) -> Result<String, Error> {
        let mut claims = Claims::create((*DEFAULT_REFRESH_INTERVAL).into());
        claims.invalid_before = self
            .0
            .expires_at
            .map(|max_age| max_age - (*DEFAULT_TIME_TOLERANCE).into());
        claims.subject = self.0.subject.as_ref().cloned();
        JwtClaims::shared_key()
            .authenticate(claims)
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Generates an access token signed with the shared secret access key.
    #[inline]
    pub fn access_token(self) -> Result<String, Error> {
        self.sign_with(JwtClaims::shared_key())
    }

    /// Generates a signature with the secret access key.
    #[inline]
    pub fn sign_with<K: MACLike>(self, key: &K) -> Result<String, Error> {
        key.authenticate(self.0)
            .map_err(|err| Error::new(err.to_string()))
    }
}

impl<T> JwtClaims<T> {
    /// Sets the nonce.
    #[inline]
    pub fn set_nonce(&mut self, nonce: impl ToString) {
        self.0.nonce = Some(nonce.to_string());
    }

    /// Returns the time the claims were created at.
    #[inline]
    pub fn issued_at(&self) -> DateTime {
        self.0
            .issued_at
            .and_then(|d| i64::try_from(d.as_micros()).ok())
            .map(DateTime::from_timestamp_micros)
            .unwrap_or_default()
    }

    /// Returns the time the claims expire at.
    #[inline]
    pub fn expires_at(&self) -> DateTime {
        self.0
            .expires_at
            .and_then(|d| i64::try_from(d.as_micros()).ok())
            .map(DateTime::from_timestamp_micros)
            .unwrap_or_default()
    }

    /// Returns the time when the claims will expire in.
    #[inline]
    pub fn expires_in(&self) -> Duration {
        self.0
            .expires_at
            .and_then(|dt| {
                dt.as_secs()
                    .checked_add_signed(-DateTime::current_timestamp())
            })
            .map(Duration::from_secs)
            .unwrap_or_default()
    }

    /// Returns the subject.
    #[inline]
    pub fn subject(&self) -> Option<&str> {
        self.0.subject.as_deref()
    }

    /// Returns the nonce.
    #[inline]
    pub fn nonce(&self) -> Option<&str> {
        self.0.nonce.as_deref()
    }

    /// Returns the custom data.
    #[inline]
    pub fn data(&self) -> &T {
        &self.0.custom
    }
}

impl JwtClaims<Map> {
    /// Adds a key-value pair to the custom data.
    #[inline]
    pub fn add_data_entry(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
        self.0.custom.upsert(key.into(), value.into());
    }
}

impl JwtClaims<()> {
    /// Returns the shared secret access key for the HMAC algorithm.
    #[inline]
    pub fn shared_key() -> &'static JwtHmacKey {
        LazyLock::force(&SECRET_KEY)
    }
}

/// Returns the default time tolerance.
#[inline]
pub(crate) fn default_time_tolerance() -> Duration {
    *DEFAULT_TIME_TOLERANCE
}

/// Returns the default verfication options.
#[inline]
pub(crate) fn default_verification_options() -> VerificationOptions {
    SHARED_VERIFICATION_OPTIONS.clone()
}

/// Shared verfications options.
static SHARED_VERIFICATION_OPTIONS: LazyLock<VerificationOptions> = LazyLock::new(|| {
    if let Some(config) = State::shared().get_config("jwt") {
        VerificationOptions {
            accept_future: config.get_bool("accept_future").unwrap_or_default(),
            required_subject: config.get_str("required-subject").map(|s| s.to_owned()),
            time_tolerance: config.get_duration("time-tolerance").map(|d| d.into()),
            max_validity: config.get_duration("max-validity").map(|d| d.into()),
            max_token_length: config.get_usize("max-token-length"),
            max_header_length: config.get_usize("max-header-length"),
            ..VerificationOptions::default()
        }
    } else {
        VerificationOptions::default()
    }
});

/// Default time tolerance.
static DEFAULT_TIME_TOLERANCE: LazyLock<Duration> = LazyLock::new(|| {
    State::shared()
        .get_config("jwt")
        .and_then(|config| config.get_duration("time-tolerance"))
        .unwrap_or_else(|| Duration::from_secs(claims::DEFAULT_TIME_TOLERANCE_SECS))
});

/// Default max age for the access token.
static DEFAULT_MAX_AGE: LazyLock<Duration> = LazyLock::new(|| {
    State::shared()
        .get_config("jwt")
        .and_then(|config| config.get_duration("max-age"))
        .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24))
});

/// Default refresh interval for the refresh token.
static DEFAULT_REFRESH_INTERVAL: LazyLock<Duration> = LazyLock::new(|| {
    State::shared()
        .get_config("jwt")
        .and_then(|config| config.get_duration("refresh-interval"))
        .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 30))
});

/// Shared secret access key for the HMAC algorithm.
static SECRET_KEY: LazyLock<JwtHmacKey> = LazyLock::new(|| {
    let config = State::shared().config();
    let checksum: [u8; 32] = config
        .get_table("jwt")
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
    let secret_key = crypto::derive_key("ZINO:JWT", &checksum);
    JwtHmacKey::from_bytes(&secret_key)
});

cfg_if::cfg_if! {
    if #[cfg(feature = "crypto-sm")] {
        use hmac::{Hmac, Mac};
        use jwt_simple::{algorithms::HMACKey, common::KeyMetadata};
        use sm3::Sm3;

        /// HMAC-SM3 key type.
        #[derive(Debug, Clone)]
        pub struct HSm3Key {
            /// key.
            key: HMACKey,
            /// Key ID.
            key_id: Option<String>,
        }

        impl HSm3Key {
            /// Creates a new instance from bytes.
            pub fn from_bytes(raw_key: &[u8]) -> Self {
                Self {
                    key: HMACKey::from_bytes(raw_key),
                    key_id: None,
                }
            }

            /// Returns the bytes.
            pub fn to_bytes(&self) -> Vec<u8> {
                self.key.to_bytes()
            }

            /// Generates a new instance with random bytes.
            pub fn generate() -> Self {
                Self {
                    key: HMACKey::generate(),
                    key_id: None,
                }
            }

            /// Sets the key ID.
            pub fn with_key_id(mut self, key_id: &str) -> Self {
                self.key_id = Some(key_id.to_owned());
                self
            }
        }

        impl MACLike for HSm3Key {
            fn jwt_alg_name() -> &'static str {
                "HSM3"
            }

            fn key(&self) -> &HMACKey {
                &self.key
            }

            fn key_id(&self) -> &Option<String> {
                &self.key_id
            }

            fn set_key_id(&mut self, key_id: String) {
                self.key_id = Some(key_id);
            }

            fn metadata(&self) -> &Option<KeyMetadata> {
                &None
            }

            fn attach_metadata(&mut self, _metadata: KeyMetadata) -> Result<(), jwt_simple::Error> {
                Ok(())
            }

            fn authentication_tag(&self, authenticated: &str) -> Vec<u8> {
                let mut mac = Hmac::<Sm3>::new_from_slice(self.key().as_ref())
                    .expect("HMAC can take key of any size");
                mac.update(authenticated.as_bytes());
                mac.finalize().into_bytes().to_vec()
            }
        }

        /// HMAC key type for JWT.
        pub type JwtHmacKey = HSm3Key;
    } else {
        /// HMAC key type for JWT.
        pub type JwtHmacKey = jwt_simple::algorithms::HS256Key;
    }
}
