//! HTTP client for Zhipu AI API.
//!
//! This module provides a client for interacting with Zhipu's AI platform,
//! including authentication and request handling.

use reqwest::{self};
use serde::{self, Deserialize};
use std::fmt::Debug;
use tracing::debug;

/// Base URL for Zhipu API.
const ZHIPU_BASE_URL: &str = "https://open.bigmodel.cn";

/// HTTP client for Zhipu AI API.
///
/// This client handles authentication and HTTP requests to Zhipu's AI platform.
#[derive(Clone)]
pub struct Client {
    /// Base URL for the API endpoint.
    pub base_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// HTTP client for making requests.
    pub http_client: reqwest::Client,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("http_client", &self.http_client)
            .field("api_key", &self.api_key)
            .finish()
    }
}

//Create a client
impl Client {
    /// Creates a new Zhipu client with the default API URL.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication.
    ///
    /// # Returns
    /// A new `Client` instance configured with the default Zhipu API URL.
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, ZHIPU_BASE_URL)
    }
    /// Creates a new Zhipu client with a custom API URL.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication.
    /// * `url` - The base URL for the Zhipu API.
    ///
    /// # Returns
    /// A new `Client` instance configured with the specified API URL.
    pub fn from_url(api_key: &str, url: &str) -> Self {
        Self {
            base_url: url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Zhipi Client Should build!"),
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
}

impl Client {
    /// Creates a POST request builder for the specified path.
    ///
    /// # Arguments
    /// * `url` - The API endpoint path.
    ///
    /// # Returns
    /// A configured `reqwest::RequestBuilder` for POST requests.
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let path = url.trim_start_matches("/");
        let url = format!("{}/{}", &self.base_url, path);
        debug!("Request URL: {}", &url);
        self.http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
    }
    /// Creates a GET request builder for the specified path.
    ///
    /// # Arguments
    /// * `url` - The API endpoint path.
    ///
    /// # Returns
    /// A configured `reqwest::RequestBuilder` for GET requests.
    #[allow(dead_code)]
    pub(crate) fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let path = url.trim_start_matches("/");
        let url = format!("{}/{}", &self.base_url, path);
        debug!("Reqwest URL: {}", &url);

        self.http_client.get(url).bearer_auth(&self.api_key)
    }
}

/// API error response from Zhipu.
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message from the API.
    pub message: String,
}

/// API response wrapper for Zhipu.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    /// Successful response with data.
    Ok(T),
    /// Error response.
    Err(ApiErrorResponse),
}
