use crate::{error::Error, extension::TomlTableExt, warn};
use openidconnect::{
    core::{CoreClient, CoreJsonWebKey, CoreProviderMetadata},
    reqwest::http_client,
    AuthType::RequestBody,
    AuthUrl, ClientId, ClientSecret, DeviceAuthorizationUrl, IntrospectionUrl, IssuerUrl,
    JsonWebKeySet, JsonWebKeySetUrl, RedirectUrl, RevocationUrl, TokenUrl, UserInfoUrl,
};
use std::ops::Deref;
use toml::Table;
use url::Url;

/// OpenID Connect client.
pub struct OidcClient(CoreClient);

impl OidcClient {
    /// Creates a new instance.
    pub fn new(
        client_id: String,
        client_secret: Option<String>,
        issuer_url: Url,
        auth_url: Url,
        token_url: Option<Url>,
        userinfo_url: Option<Url>,
        keys: Vec<CoreJsonWebKey>,
    ) -> Self {
        let client_id = ClientId::new(client_id);
        let client_secret = client_secret.map(ClientSecret::new);
        let issuer_url = IssuerUrl::from_url(issuer_url);
        let auth_url = AuthUrl::from_url(auth_url);
        let token_url = token_url.map(TokenUrl::from_url);
        let userinfo_url = userinfo_url.map(UserInfoUrl::from_url);
        let jwks = JsonWebKeySet::new(keys);
        let client = CoreClient::new(
            client_id,
            client_secret,
            issuer_url,
            auth_url,
            token_url,
            userinfo_url,
            jwks,
        );
        Self(client.set_auth_type(RequestBody))
    }

    /// Attempts to create a new instance from provider metadata.
    pub fn try_from_provider_metadata(
        client_id: String,
        client_secret: Option<String>,
        issuer: String,
    ) -> Result<Self, Error> {
        let issuer_url = IssuerUrl::new(issuer)?;
        let provider_metadata = CoreProviderMetadata::discover(&issuer_url, http_client)?;
        let client_id = ClientId::new(client_id);
        let client_secret = client_secret.map(ClientSecret::new);
        let client =
            CoreClient::from_provider_metadata(provider_metadata, client_id, client_secret);
        Ok(Self(client.set_auth_type(RequestBody)))
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
        let issuer_url = config
            .get_str("issuer-url")
            .ok_or_else(|| warn!("the `issuer-url` field should be specified"))?
            .parse()
            .map(IssuerUrl::from_url)?;
        let mut client = if let Some(jwks_url) = config.get_str("jwks-url") {
            let jwks_url = JsonWebKeySetUrl::new(jwks_url.to_owned())?;
            let jwks = JsonWebKeySet::fetch(&jwks_url, http_client)?;
            let auth_url = config
                .get_str("auth-url")
                .ok_or_else(|| warn!("the `auth-url` field should be specified"))?
                .parse()
                .map(AuthUrl::from_url)?;
            let token_url = config
                .get_str("token-url")
                .map(|s| TokenUrl::new(s.to_owned()))
                .transpose()?;
            let userinfo_url = config
                .get_str("userinfo-url")
                .map(|s| UserInfoUrl::new(s.to_owned()))
                .transpose()?;
            CoreClient::new(
                client_id,
                client_secret,
                issuer_url,
                auth_url,
                token_url,
                userinfo_url,
                jwks,
            )
        } else {
            let provider_metadata = CoreProviderMetadata::discover(&issuer_url, http_client)?;
            CoreClient::from_provider_metadata(provider_metadata, client_id, client_secret)
        };
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
            client = client.set_device_authorization_uri(url);
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
        Self(self.0.set_device_authorization_uri(url))
    }
}

impl Deref for OidcClient {
    type Target = CoreClient;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
