//! HTTP client for MiniMax AI API.
//!
//! This module provides a client for interacting with MiniMax's AI platform,
//! including authentication and request handling.

/// Model identifier for MiniMax Text-01.
pub const MINIMAX_TEXT_01: &str = "MiniMax-Text-01";

/// Base URL for MiniMax API.
const MINIMAX_API_BASE_URL: &str = "https://api.minimaxi.com";

/// HTTP client for MiniMax AI API.
///
/// This client handles authentication and HTTP requests to MiniMax's AI platform.
/// It supports both standard and streaming completions.
#[derive(Clone)]
pub struct Client {
    /// Base URL for the API endpoint.
    base_url: String,
    /// API key for authentication.
    api_key: String,
    /// HTTP client for making requests.
    http_client: reqwest::Client,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("http_client", &"<reqwest::Client>")
            .field("api_key", &"<REDACTED>")
            .finish()
    }
}

impl Client {
    /// Creates a new MiniMax client with the default API URL.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication.
    ///
    /// # Returns
    /// A new `Client` instance configured with the default MiniMax API URL.
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, MINIMAX_API_BASE_URL)
    }

    /// Creates a new MiniMax client with a custom API URL.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication.
    /// * `base_url` - The base URL for the MiniMax API.
    ///
    /// # Returns
    /// A new `Client` instance configured with the specified API URL.
    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("MiniMax reqwest client should build"),
        }
    }

    /// Configures the client with a custom HTTP client.
    ///
    /// # Arguments
    /// * `client` - A custom `reqwest::Client` instance.
    ///
    /// # Returns
    /// The `Client` instance with the custom HTTP client configured.
    pub fn with_custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// Creates a POST request builder for the specified path.
    ///
    /// # Arguments
    /// * `path` - The API endpoint path.
    ///
    /// # Returns
    /// A configured `reqwest::RequestBuilder` for POST requests.
    pub(crate) fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let path = path.trim_start_matches('/');
        let url = format!("{}/{}", self.base_url, path);
        self.http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
    }

    /// Creates a GET request builder for the specified path.
    ///
    /// # Arguments
    /// * `path` - The API endpoint path.
    ///
    /// # Returns
    /// A configured `reqwest::RequestBuilder` for GET requests.
    #[allow(dead_code)]
    pub(crate) fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let path = path.trim_start_matches('/');
        let url = format!("{}/{}", self.base_url, path);
        self.http_client
            .get(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
    }
}
