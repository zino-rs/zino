use reqwest;
use serde::de::DeserializeOwned;
use serde::{self, Deserialize};
use tracing::debug;
const BAIDU_QIANFAN_URL: &str = "https://qianfan.baidubce.com";

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
    pub fn from_api_key(api_key: &str) -> Self {
        Self::from_url(api_key, BAIDU_QIANFAN_URL)
    }

    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Baidu reqwest should be build!"),
        }
    }

    pub fn with_custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// send post request to the specified path
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

    /// send get request to the specified path
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
pub struct ApiErrorResponse {
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}
