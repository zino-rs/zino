use reqwest::{self};
use serde::{self, Deserialize};
use std::fmt::Debug;

const ZHIPU_BASE_URL: &str = "https://open.bigmodel.cn";

#[derive(Clone)]
pub struct Client {
    pub base_url: String,
    pub api_key: String,
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
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, ZHIPU_BASE_URL)
    }
    pub fn from_url(api_key: &str, url: &str) -> Self {
        Self {
            base_url: url.to_string(),
            api_key: api_key.to_string(),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Zhipi Client Should build!"),
        }
    }

    pub fn with_custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;

        self
    }
}

impl Client {
    //发送http请求
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let path = url.trim_start_matches("/");
        let url = format!("{}/{}", &self.base_url, path);
        println!("Request URL: {}", &url);
        self.http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
    }
    //
    pub(crate) fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let path = url.trim_start_matches("/");
        let url = format!("{}/{}", &self.base_url, path);
        println!("Reqwest URL: {}", &url);

        self.http_client.get(url).bearer_auth(&self.api_key)
    }

}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}
