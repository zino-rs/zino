use crate::{extension::TomlTableExt, state::State, Map};
use jwt_simple::{
    algorithms::MACLike,
    claims::{self, Claims, JWTClaims},
    common::VerificationOptions,
};
use std::{sync::LazyLock, time::Duration};

/// JWT Claims.
pub struct JwtClaims(pub(crate) JWTClaims<Map>);

impl JwtClaims {
    /// Creates a new instance.
    #[inline]
    pub fn new(claims: Map, valid_for: Duration) -> Self {
        let claims = Claims::with_custom_claims(claims, valid_for.into());
        Self(claims)
    }

    /// Creates a new instance with the nonce.
    #[inline]
    pub fn with_nonce(claims: Map, valid_for: Duration, nonce: String) -> Self {
        let claims = Claims::with_custom_claims(claims, valid_for.into()).with_nonce(nonce);
        Self(claims)
    }

    /// Generates a signature with the secret access key.
    #[inline]
    pub fn sign_with<K: MACLike>(self, key: &K) -> Result<String, jwt_simple::Error> {
        key.authenticate(self.0)
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
    let config = State::shared().config();
    VerificationOptions {
        accept_future: config.get_bool("accept_future").unwrap_or_default(),
        required_subject: config.get_str("required-subject").map(|s| s.to_owned()),
        time_tolerance: config.get_duration("time-tolerance").map(|d| d.into()),
        max_validity: config.get_duration("max-validity").map(|d| d.into()),
        max_token_length: config.get_usize("max_token_length"),
        max_header_length: config.get_usize("max_header_length"),
        ..VerificationOptions::default()
    }
});

/// Default time tolerance.
static DEFAULT_TIME_TOLERANCE: LazyLock<Duration> = LazyLock::new(|| {
    State::shared()
        .config()
        .get_duration("time-tolerance")
        .unwrap_or_else(|| Duration::from_secs(claims::DEFAULT_TIME_TOLERANCE_SECS))
});
