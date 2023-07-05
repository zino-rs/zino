use crate::{
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    state::State,
    JsonValue, Map,
};
use jwt_simple::{
    algorithms::{HS256Key, MACLike},
    claims::{self, Claims, JWTClaims},
    common::VerificationOptions,
};
use std::{sync::LazyLock, time::Duration};

/// JWT Claims.
pub struct JwtClaims(pub(crate) JWTClaims<Map>);

impl JwtClaims {
    /// Creates a new instance.
    #[inline]
    pub fn new(subject: impl ToString) -> Self {
        let mut claims = Self::default();
        claims.0.subject = Some(subject.to_string());
        claims
    }

    /// Sets the nonce.
    #[inline]
    pub fn set_nonce(&mut self, nonce: impl ToString) {
        self.0.nonce = Some(nonce.to_string());
    }

    /// Adds a key-value pair to the custom data.
    #[inline]
    pub fn add_data_entry(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
        self.0.custom.upsert(key.into(), value.into());
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
    pub fn data(&self) -> &Map {
        &self.0.custom
    }

    /// Generates a signature with the secret access key.
    #[inline]
    pub fn sign_with<K: MACLike>(self, key: &K) -> Result<String, Error> {
        key.authenticate(self.0)
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Returns the shared secret access key for the `HS256` JWT algorithm.
    #[inline]
    pub fn shared_key() -> &'static HS256Key {
        LazyLock::force(&SECRET_KEY)
    }
}

impl Default for JwtClaims {
    #[inline]
    fn default() -> Self {
        let mut claims = Claims::with_custom_claims(Map::new(), (*DEFAULT_MAX_AGE).into());
        claims.invalid_before = None;
        Self(claims)
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
            max_token_length: config.get_usize("max_token_length"),
            max_header_length: config.get_usize("max_header_length"),
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

/// Default max age.
static DEFAULT_MAX_AGE: LazyLock<Duration> = LazyLock::new(|| {
    State::shared()
        .get_config("jwt")
        .and_then(|config| config.get_duration("max-age"))
        .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24))
});

/// Shared secret access key for the `HS256` JWT algorithm.
static SECRET_KEY: LazyLock<HS256Key> = LazyLock::new(|| {
    if let Some(secret_key) = crate::application::SECRET_KEY.get() {
        HS256Key::from_bytes(secret_key)
    } else {
        HS256Key::generate()
    }
});
