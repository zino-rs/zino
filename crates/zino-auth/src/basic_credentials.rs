use serde::{Deserialize, Serialize};

/// Credentials for the HTTP basic authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicCredentials {
    /// Username.
    #[serde(alias = "account")]
    username: String,
    /// Password.
    password: String,
}

impl BasicCredentials {
    /// Creates a new instance.
    #[inline]
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Returns the username.
    #[inline]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the password.
    #[inline]
    pub fn password(&self) -> &str {
        &self.password
    }
}
