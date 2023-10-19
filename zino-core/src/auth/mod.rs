//! Authentication and authorization.
//!
//! ## Feature flags
//!
//! The following optional features are available:
//!
//! | Name          | Description                                          | Default? |
//! |---------------|------------------------------------------------------|----------|
//! | `auth-oauth2` | Enables the integration with [`oauth2`].             | No       |
//! | `auth-oidc`   | Enables the integration with [`openidconnect`].      | No       |
//! | `auth-totp`   | Enables the integration with [`totp-rs`].            | No       |
//!
//! [`oauth2`]: https://crates.io/crates/oauth2
//! [`openidconnect`]: https://crates.io/crates/openidconnect
//! [`totp-rs`]: https://crates.io/crates/totp-rs

mod access_key;
mod authentication;
mod authorization_provider;
mod client_credentials;
mod jwt_claims;
mod security_token;
mod session_id;
mod user_session;

#[cfg(feature = "auth-oauth2")]
mod oauth2_client;

#[cfg(feature = "auth-oidc")]
mod oidc_client;

pub(crate) use jwt_claims::{default_time_tolerance, default_verification_options};
pub(crate) use security_token::ParseSecurityTokenError;

pub use access_key::{AccessKeyId, SecretAccessKey};
pub use authentication::Authentication;
pub use authorization_provider::AuthorizationProvider;
pub use client_credentials::ClientCredentials;
pub use jwt_claims::{JwtClaims, JwtHmacKey};
pub use security_token::SecurityToken;
pub use session_id::SessionId;
pub use user_session::UserSession;

#[cfg(feature = "auth-oauth2")]
pub use oauth2_client::OAuth2Client;

#[cfg(feature = "auth-oidc")]
pub use oidc_client::OidcClient;
