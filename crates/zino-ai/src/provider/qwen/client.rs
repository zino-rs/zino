//! Qwen API Client
use crate::client::ProviderClient;
use serde::{Deserialize, Serialize};
// ================================================================
// Main Qwen Client
// ================================================================
const QWEN_API_BASE_URL: &str = "https://dashscope.aliyuncs.com";

#[derive(Clone)]
pub struct Client {
    pub base_url: String,
    pub api_key: String,
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
    /// Create a new Qwen client with the given API key.
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, QWEN_API_BASE_URL)
    }

    /// Create a new Qwen client with the given API key and base API URL.
    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Qwen reqwest client should build"),
        }
    }

    /// Use your own `reqwest::Client`.
    /// The required headers will be automatically attached upon trying to make a request.
    pub fn with_custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    pub(crate) fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path).replace("//", "/");
        self.http_client.post(url).bearer_auth(&self.api_key)
    }

    pub(crate) fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path).replace("//", "/");
        self.http_client.get(url).bearer_auth(&self.api_key)
    }

    // pub fn agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
    //     AgentBuilder::new(self.completion_model(model))
    // }

    // /// Create an extractor builder with the given completion model.
    // pub fn extractor<T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync>(
    //     &self,
    //     model: &str,
    // ) -> ExtractorBuilder<T, CompletionModel> {
    //     ExtractorBuilder::new(self.completion_model(model))
    // }
}

impl ProviderClient for Client {
    /// Create a new Qwen client from the `QWEN_API_KEY` environment variable.
    /// Panics if the environment variable is not set.
    fn from_env() -> Self {
        let api_key = std::env::var("QWEN_API_KEY").expect("QWEN_API_KEY not set");
        Self::new(&api_key)
    }

    fn from_val(input: String) -> Self
    where
        Self: Sized,
    {
        let api_key = input;
        Self::new(&api_key)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub(crate) code: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub image_count: Option<u32>,
}
