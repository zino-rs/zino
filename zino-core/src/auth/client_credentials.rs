use super::AuthorizationProvider;
use crate::{
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    Map, SharedString,
};
use parking_lot::RwLock;
use std::{marker::PhantomData, time::Duration};
use toml::Table;

/// Credentials for the client authentication.
#[derive(Debug)]
pub struct ClientCredentials<S: ?Sized> {
    /// Client ID.
    client_id: SharedString,
    /// Client key.
    client_key: SharedString,
    /// Client secret.
    client_secret: SharedString,
    /// Access token.
    access_token: RwLock<String>,
    /// Expires time.
    expires_at: RwLock<DateTime>,
    /// Phantom type of authorization server.
    phantom: PhantomData<S>,
}

impl<S: ?Sized> ClientCredentials<S> {
    /// Creates a new instance.
    #[inline]
    pub fn new(client_id: impl Into<SharedString>, client_secret: impl Into<SharedString>) -> Self {
        Self {
            client_id: client_id.into(),
            client_key: "".into(),
            client_secret: client_secret.into(),
            access_token: RwLock::new(String::new()),
            expires_at: RwLock::new(DateTime::now()),
            phantom: PhantomData,
        }
    }

    /// Attempts to create a new instance with the configuration.
    #[inline]
    pub fn try_from_config(config: &'static Table) -> Result<Self, Error> {
        let client_id = config
            .get_str("client-id")
            .ok_or_else(|| Error::new("the `client-id` field should be specified"))?;
        let client_key = config.get_str("client-key").unwrap_or_default();
        let client_secret = config
            .get_str("client-secret")
            .ok_or_else(|| Error::new("the `client-secret` field should be specified"))?;
        Ok(Self {
            client_id: client_id.into(),
            client_key: client_key.into(),
            client_secret: client_secret.into(),
            access_token: RwLock::new(String::new()),
            expires_at: RwLock::new(DateTime::now()),
            phantom: PhantomData,
        })
    }

    /// Sets the client key.
    #[inline]
    pub fn set_client_key(&mut self, client_key: impl Into<SharedString>) {
        self.client_key = client_key.into();
    }

    /// Sets the access token.
    #[inline]
    pub fn set_access_token(&self, access_token: impl ToString) {
        *self.access_token.write() = access_token.to_string();
    }

    /// Sets the expires.
    #[inline]
    pub fn set_expires(&self, expires_in: Duration) {
        *self.expires_at.write() = DateTime::now() + expires_in
    }

    /// Returns the client ID.
    #[inline]
    pub fn client_id(&self) -> &str {
        self.client_id.as_ref()
    }

    /// Returns the client key.
    #[inline]
    pub fn client_key(&self) -> &str {
        self.client_key.as_ref()
    }

    /// Returns the client secret.
    #[inline]
    pub fn client_secret(&self) -> &str {
        self.client_secret.as_ref()
    }

    /// Returns the access token regardless of whether it has been expired.
    #[inline]
    pub fn access_token(&self) -> String {
        self.access_token.read().clone()
    }

    /// Returns the time the client credentials expire at.
    #[inline]
    pub fn expires_at(&self) -> DateTime {
        *self.expires_at.read()
    }

    /// Returns `true` if the access token for the client credentials has been expired.
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.expires_at() <= DateTime::now()
    }

    /// Converts `self` to the request params.
    pub fn to_request_params(&self) -> Map {
        let mut params = Map::new();
        let client_id = self.client_id();
        let client_key = self.client_key();
        let client_secret = self.client_secret();
        if !client_id.is_empty() {
            params.upsert("client_id", client_id);
        }
        if !client_key.is_empty() {
            params.upsert("client_key", client_key);
        }
        if !client_secret.is_empty() {
            params.upsert("client_secret", client_secret);
        }
        params
    }
}

impl<S: ?Sized + AuthorizationProvider> ClientCredentials<S> {
    /// Requests an access token for the client credentials.
    #[inline]
    pub async fn request(&self) -> Result<String, Error> {
        if self.is_expired() {
            S::grant_client_credentials(self).await?;
        }
        Ok(self.access_token())
    }
}
