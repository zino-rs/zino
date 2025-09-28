//! HTTP client for Baidu AI API.
//!
//! This module provides a client for interacting with Baidu's Qianfan AI platform,
//! including authentication and request handling.

use reqwest;
use serde::de::DeserializeOwned;
use serde::{self, Deserialize};
use tracing::debug;

/// Base URL for Baidu Qianfan API.
const BAIDU_QIANFAN_URL: &str = "https://qianfan.baidubce.com";

/// HTTP client for Baidu AI API.
///
/// This client handles authentication and HTTP requests to Baidu's Qianfan platform.
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
    /// Creates a new client with the default Baidu Qianfan API URL.
    ///
    /// # Arguments
    /// * `api_key` - Your Baidu API key for authentication.
    ///
    /// # Returns
    /// A new `Client` instance configured for Baidu Qianfan.
    pub fn from_api_key(api_key: &str) -> Self {
        Self::from_url(api_key, BAIDU_QIANFAN_URL)
    }

    /// Creates a new client with a custom base URL.
    ///
    /// # Arguments
    /// * `api_key` - Your API key for authentication.
    /// * `base_url` - Custom base URL for the API endpoint.
    ///
    /// # Returns
    /// A new `Client` instance with the specified configuration.
    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Baidu reqwest should be build!"),
        }
    }

    /// Configures the client with a custom HTTP client.
    ///
    /// # Arguments
    /// * `client` - Custom reqwest client instance.
    ///
    /// # Returns
    /// The client instance with the custom HTTP client configured.
    pub fn with_custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// Sends a POST request to the specified path.
    ///
    /// # Arguments
    /// * `path` - The API endpoint path (leading slash will be removed).
    ///
    /// # Returns
    /// A configured `RequestBuilder` with authentication and headers.
    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let path = path.trim_start_matches('/'); // remove the starting slash
        let url = format!("{}/{}", self.base_url, path);
        debug!(target: "baidu_client", url=%url, method="POST");
        self.http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
    }

    /// Sends a GET request to the specified path.
    ///
    /// # Arguments
    /// * `path` - The API endpoint path (leading slash will be removed).
    ///
    /// # Returns
    /// A configured `RequestBuilder` with authentication and headers.
    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let path = path.trim_start_matches('/'); // 移除路径开头的斜杠
        let url = format!("{}/{}", self.base_url, path);
        debug!(target: "baidu_client", url=%url, method="GET");
        self.http_client
            .get(url)
            .bearer_auth(&self.api_key)
            .header("Accept", "application/json")
    }

    /// send post request and catch feedback
    pub async fn post_json<T>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let response = self.post(path).json(body).send().await?;

        response.json::<T>().await
    }

    /// send get request and catch feedback
    pub async fn get_json<T>(&self, path: &str) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let response = self.get(path).send().await?;

        response.json::<T>().await
    }

    /// send post request
    pub async fn post_raw(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.post(path).json(body).send().await
    }

    /// send get request to the specified path
    pub async fn get_raw(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.get(path).send().await
    }
}

#[derive(Debug, Deserialize)]
/// API error response from Baidu.
pub struct ApiErrorResponse {
    /// Error message from the API.
    #[allow(dead_code)]
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
/// API response wrapper for Baidu.
pub(crate) enum ApiResponse<T> {
    /// Successful response with data.
    Ok(T),
    /// Error response.
    #[allow(dead_code)]
    Err(ApiErrorResponse),
}
