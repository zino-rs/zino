use super::client::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RerankerError {
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("RequestError: {0}")]
    RequestError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("ResponseError: {0}")]
    ResponseError(String),

    #[error("ProviderError: {0}")]
    ProviderError(String),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RerankerRequest {
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<usize>,
    pub user: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RerankerResultItem {
    pub document: String,
    pub relevance_score: f64,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RerankerUsage {
    pub prompt_tokens: Option<usize>,
    pub prompt_tokens_details: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RerankerResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<usize>,
    pub model: Option<String>,
    pub results: Option<Vec<RerankerResultItem>>,
    pub usage: Option<RerankerUsage>,
}

#[derive(Debug)]
pub struct RerankerModel {
    //embedding-v1
    pub model: String,
    pub client: Client,
}

impl RerankerModel {
    pub fn new(model: &str, client: Client) -> Self {
        Self {
            model: model.to_string(),
            client,
        }
    }

    fn create_request_body(&self, request: RerankerRequest) -> Result<Value, RerankerError> {
        let mut request_body = serde_json::to_value(request)?;
        request_body["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_body)
    }

    pub async fn rerank(
        &self,
        request: RerankerRequest,
    ) -> Result<RerankerResponse, RerankerError> {
        let request_json = self.create_request_body(request)?;

        let response = self
            .client
            .post("/v2/rerank")
            .json(&request_json)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(RerankerError::ProviderError(format!(
                "HTTP {}: {}",
                status.as_u16(),
                text
            )));
        }

        let text = response.text().await?;
        if text.is_empty() {
            return Err(RerankerError::ResponseError("Empty response".to_string()));
        }
        let parsed: RerankerResponse = serde_json::from_str(&text)?;
        Ok(parsed)
    }
}
