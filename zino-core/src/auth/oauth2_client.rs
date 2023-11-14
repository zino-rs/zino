use crate::{error::Error, extension::TomlTableExt, warn};
use oauth2::{
    basic::BasicClient, AuthType::RequestBody, AuthUrl, ClientId, ClientSecret,
    DeviceAuthorizationUrl, IntrospectionUrl, RedirectUrl, RevocationUrl, TokenUrl,
};
use std::ops::Deref;
use toml::Table;
use url::Url;

/// OAuth2 client.
pub struct OAuth2Client(BasicClient);

impl OAuth2Client {
    /// Creates a new instance.
    pub fn new(
        client_id: String,
        client_secret: Option<String>,
        auth_url: Url,
        token_url: Option<Url>,
    ) -> Self {
        let client_id = ClientId::new(client_id);
        let client_secret = client_secret.map(ClientSecret::new);
        let auth_url = AuthUrl::from_url(auth_url);
        let token_url = token_url.map(TokenUrl::from_url);
        let client = BasicClient::new(client_id, client_secret, auth_url, token_url);
        Self(client.set_auth_type(RequestBody))
    }

    /// Attempts to create a new instance with the configuration.
    pub fn try_from_config(config: &Table) -> Result<Self, Error> {
        let client_id = config
            .get_str("client-id")
            .map(|s| ClientId::new(s.to_owned()))
            .ok_or_else(|| warn!("the `client-id` field should be specified"))?;
        let client_secret = config
            .get_str("client-secret")
            .map(|s| ClientSecret::new(s.to_owned()));
        let auth_url = config
            .get_str("auth-url")
            .ok_or_else(|| warn!("the `auth-url` field should be specified"))?
            .parse()
            .map(AuthUrl::from_url)?;
        let token_url = config
            .get_str("token-url")
            .map(|s| TokenUrl::new(s.to_owned()))
            .transpose()?;
        let mut client = BasicClient::new(client_id, client_secret, auth_url, token_url);
        if let Some(redirect_url) = config.get_str("redirect-url") {
            let url = RedirectUrl::new(redirect_url.to_owned())?;
            client = client.set_redirect_uri(url);
        }
        if let Some(introspection_url) = config.get_str("introspection-url") {
            let url = IntrospectionUrl::new(introspection_url.to_owned())?;
            client = client.set_introspection_uri(url);
        }
        if let Some(revocation_url) = config.get_str("revocation-url") {
            let url = RevocationUrl::new(revocation_url.to_owned())?;
            client = client.set_revocation_uri(url);
        }
        if let Some(device_authorization_url) = config.get_str("device-authorization-url") {
            let url = DeviceAuthorizationUrl::new(device_authorization_url.to_owned())?;
            client = client.set_device_authorization_url(url);
        }
        Ok(Self(client.set_auth_type(RequestBody)))
    }

    /// Sets the redirect URL used by the authorization endpoint.
    #[inline]
    pub fn with_redirect_uri(self, redirect_url: Url) -> Self {
        let url = RedirectUrl::from_url(redirect_url);
        Self(self.0.set_redirect_uri(url))
    }

    /// Sets the introspection URL for contacting the introspection endpoint
    /// ([RFC 7662](https://tools.ietf.org/html/rfc7662)).
    #[inline]
    pub fn with_introspection_uri(self, introspection_url: Url) -> Self {
        let url = IntrospectionUrl::from_url(introspection_url);
        Self(self.0.set_introspection_uri(url))
    }

    /// Sets the revocation URL for contacting the revocation endpoint
    /// ([RFC 7009](https://tools.ietf.org/html/rfc7009)).
    #[inline]
    pub fn with_revocation_uri(self, revocation_url: Url) -> Self {
        let url = RevocationUrl::from_url(revocation_url);
        Self(self.0.set_revocation_uri(url))
    }

    /// Sets the the device authorization URL used by the device authorization endpoint.
    /// Used for Device Code Flow, as per [RFC 8628](https://tools.ietf.org/html/rfc8628).
    #[inline]
    pub fn with_device_authorization_uri(self, device_authorization_url: Url) -> Self {
        let url = DeviceAuthorizationUrl::from_url(device_authorization_url);
        Self(self.0.set_device_authorization_url(url))
    }
}

impl Deref for OAuth2Client {
    type Target = BasicClient;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
