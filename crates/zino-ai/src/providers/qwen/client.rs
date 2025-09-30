//! HTTP client for Qwen AI API.
//!
//! This module provides a client for interacting with Alibaba's Qwen AI platform,
//! including authentication and request handling.

use serde::{Deserialize, Serialize};

/// Base URL for Qwen API.
const QWEN_API_BASE_URL: &str = "https://dashscope.aliyuncs.com";

/// HTTP client for Qwen AI API.
///
/// This client handles authentication and HTTP requests to Alibaba's Qwen AI platform.
/// It supports both standard and streaming completions.
#[derive(Clone)]
pub struct Client {
    /// Base URL for the API endpoint.
    pub base_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// HTTP client for making requests.
    pub http_client: reqwest::Client,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("http_client", &self.http_client)
            .field("api_key", &"<REDACTED>")
            .finish()
    }
}

impl Client {
    /// Creates a new Qwen client with the default API URL.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication.
    ///
    /// # Returns
    /// A new `Client` instance configured with the default Qwen API URL.
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, QWEN_API_BASE_URL)
    }

    /// Creates a new Qwen client with a custom API URL.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication.
    /// * `base_url` - The base URL for the Qwen API.
    ///
    /// # Returns
    /// A new `Client` instance configured with the specified API URL.
    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Qwen reqwest client should build"),
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
        let url = format!("{}/{}", self.base_url, path).replace("//", "/");
        self.http_client.post(url).bearer_auth(&self.api_key)
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
        let url = format!("{}/{}", self.base_url, path).replace("//", "/");
        self.http_client.get(url).bearer_auth(&self.api_key)
    }
}

/// API error response from Qwen.
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message from the API.
    pub(crate) message: String,
}

/// API response wrapper for Qwen.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub(crate) enum ApiResponse<T> {
    /// Successful response with data.
    Ok(T),
    /// Error response.
    #[allow(dead_code)]
    Err(ApiErrorResponse),
}

/// Usage information for Qwen API responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUsage {
    /// Number of input tokens.
    pub input_tokens: Option<u32>,
    /// Number of output tokens.
    pub output_tokens: Option<u32>,
    /// Total number of tokens used.
    pub total_tokens: Option<u32>,
    /// Number of images processed.
    pub image_count: Option<u32>,
}
