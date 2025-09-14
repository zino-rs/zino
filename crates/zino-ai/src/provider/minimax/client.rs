use crate::client::ProviderClient;
pub const MINIMAX_TEXT_01: &str = "MiniMax-Text-01";

const MINIMAX_API_BASE_URL: &str = "https://api.minimaxi.com";

#[derive(Clone)]
pub struct Client {
    base_url: String,
    api_key: String,
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
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, MINIMAX_API_BASE_URL)
    }

    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("MiniMax reqwest client should build"),
        }
    }

    pub fn with_custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    pub(crate) fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let path = path.trim_start_matches('/');
        let url = format!("{}/{}", self.base_url, path);
        self.http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
    }

    pub(crate) fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let path = path.trim_start_matches('/');
        let url = format!("{}/{}", self.base_url, path);
        self.http_client
            .get(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
    }
}

impl ProviderClient for Client {
    fn from_env() -> Self
    where
        Self: Sized,
    {
        let api_key = std::env::var("MINIMAX_API_KEY").expect("MINIMAX_API_KEY not set");
        Self::new(&api_key)
    }

    fn from_val(input: String) -> Self
    where
        Self: Sized,
    {
        Self::new(&input)
    }
}
