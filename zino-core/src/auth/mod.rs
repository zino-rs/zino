//! Authentication and authorization.

mod access_key;
mod authentication;
mod authorization_provider;
mod client_credentials;
mod security_token;
mod session_id;
mod user_session;

pub use access_key::{AccessKeyId, SecretAccessKey};
pub use authentication::Authentication;
pub use authorization_provider::AuthorizationProvider;
pub use client_credentials::ClientCredentials;
pub use security_token::{ParseSecurityTokenError, SecurityToken};
pub use session_id::{ParseSessionIdError, SessionId};
pub use user_session::UserSession;

#[cfg(feature = "jwt")]
mod jwt_claims;
#[cfg(feature = "opa")]
mod rego_engine;

#[cfg(feature = "jwt")]
pub use jwt_claims::{default_time_tolerance, default_verification_options, JwtClaims, JwtHmacKey};

#[cfg(feature = "opa")]
pub use rego_engine::RegoEngine;
