// ================================================================
// Qwen Completion API
// ================================================================
use super::{ApiErrorResponse, Client};
//Rig has thes crate, but it doesn't expose them, so we copy them here.
use crate::completions::{self, CompletionError, CompletionRequest, Message};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
pub mod streaming;

impl From<ApiErrorResponse> for CompletionError {
    fn from(err: ApiErrorResponse) -> Self {
        CompletionError::ProviderError(err.message)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    pub logprobs: Option<serde_json::Value>,
    pub finish_reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

impl fmt::Display for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Usage {
            prompt_tokens,
            total_tokens,
        } = self;
        write!(
            f,
            "Prompt tokens: {prompt_tokens} Total tokens: {total_tokens}"
        )
    }
}

#[derive(Clone)]
pub struct CompletionModel {
    pub(crate) client: Client,
    pub model: String,
}

impl CompletionModel {
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    pub(crate) fn create_completion_request(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<Value, CompletionError> {
        // Build up the order of messages (context, chat_history)
        let mut request_json = serde_json::to_value(&completion_request)?;
        request_json["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_json)
    }
}

impl completions::CompletionModel for CompletionModel {
    type Response = serde_json::Value;
    type StreamingResponse = super::completion::streaming::QwenStreamingCompletionResponse;

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<serde_json::Value, CompletionError> {
        let request = self.create_completion_request(completion_request)?;

        let response = self
            .client
            .post("/compatible-mode/v1/chat/completions")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let t = response.text().await?;
            tracing::debug!(target: "Zino-ai", "Qwen completion response: {}", t);

            if t.is_empty() {
                return Err(CompletionError::ProviderError(
                    "Empty response from Qwen API".to_string(),
                ));
            }

            let parsed: CompletionResponse = serde_json::from_str(&t).map_err(|parse_err| {
                CompletionError::ProviderError(format!(
                    "Failed to parse response: {}. Response was: {}",
                    parse_err, t
                ))
            })?;

            let raw_response = serde_json::to_value(&parsed).map_err(|e| {
                CompletionError::ProviderError(format!("Failed to serialize response: {}", e))
            })?;

            Ok(raw_response)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(CompletionError::ProviderError(format!(
                "HTTP {}: {}",
                status, error_text
            )))
        }
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<
        crate::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
        CompletionError,
    > {
        let mut request_body = self.create_completion_request(request)?;

        // Add streaming parameter for Qwen API
        request_body["stream"] = serde_json::Value::Bool(true);

        // Add stream_options to include usage information in the final chunk
        request_body["stream_options"] = serde_json::json!({
            "include_usage": true
        });

        let request_builder = self
            .client
            .post("/compatible-mode/v1/chat/completions")
            .json(&request_body);

        super::completion::streaming::send_compatible_streaming_request(request_builder).await
    }
}
