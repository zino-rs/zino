use crate::{
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    state::State,
    JsonValue, Map,
};
use hkdf::Hkdf;
use jwt_simple::{
    algorithms::{HS256Key, MACLike},
    claims::{self, Claims, JWTClaims},
    common::VerificationOptions,
};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::{env, sync::LazyLock, time::Duration};

/// JWT Claims.
#[derive(Debug, Clone)]
pub struct JwtClaims<T = Map>(pub(crate) JWTClaims<T>);

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

impl JwtClaims<Map> {
    /// Adds a key-value pair to the custom data.
    #[inline]
    pub fn add_data_entry(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
        self.0.custom.upsert(key.into(), value.into());
    }
}

impl JwtClaims<()> {
    /// Returns the shared secret access key for the `HS256` JWT algorithm.
    #[inline]
    pub fn shared_key() -> &'static HS256Key {
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

/// Shared secret access key for the `HS256` JWT algorithm.
static SECRET_KEY: LazyLock<HS256Key> = LazyLock::new(|| {
    let config = State::shared().config();
    let checksum: [u8; 32] = config
        .get_table("jwt")
        .and_then(|t| t.get_str("checksum"))
        .and_then(|checksum| checksum.as_bytes().try_into().ok())
        .unwrap_or_else(|| {
            let app_name = config
                .get_str("name")
                .map(|s| s.to_owned())
                .unwrap_or_else(|| {
                    env::var("CARGO_PKG_NAME")
                        .expect("fail to get the environment variable `CARGO_PKG_NAME`")
                });
            let mut hasher = Sha256::new();
            hasher.update(app_name.as_bytes());
            hasher.finalize().into()
        });

    let mut secret_key = [0; 64];
    let info = "ZINO:JWT;CHECKSUM:SHA256;HKDF:HMAC-SHA256";
    Hkdf::<Sha256>::from_prk(&checksum)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut secret_key)
        .expect("invalid length for Sha256 to output");
    HS256Key::from_bytes(&secret_key)
});
