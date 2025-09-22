use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::Client;
use crate::completions::{self, CompletionError, CompletionRequest, Message};
use crate::streaming::{EmptyResponse, RawStreamingChoice, StreamingCompletionResponse};
use async_stream::stream;
use futures::StreamExt;
use reqwest::RequestBuilder;
use tracing::debug;

// ================================================================
// MiniMax Completion API
// Docs (summary from user): https://api.minimaxi.com/v1/text/chatcompletion_v2
// ================================================================

// MiniMax Streaming Response Structures
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingChoice {
    index: usize,
    #[serde(default)]
    delta: Option<MiniMaxStreamingDelta>,
    #[serde(default)]
    message: Option<MiniMaxStreamingMessage>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingMessage {
    content: String,
    role: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingUsage {
    total_tokens: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingChunk {
    id: String,
    choices: Vec<MiniMaxStreamingChoice>,
    created: u64,
    model: String,
    object: String,
    #[serde(default)]
    usage: Option<MiniMaxStreamingUsage>,
}

pub const MINIMAX_M1: &str = "MiniMax-M1";
pub const MINIMAX_TEXT_01: &str = "MiniMax-Text-01";

// ================================================================
// Request/Response Models (MiniMax specific)
// ================================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
    #[serde(default)]
    pub base_resp: Option<BaseResp>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseResp {
    pub status_code: i32,
    #[serde(default)]
    pub status_msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    #[serde(default)]
    pub finish_reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

// ================================================================
// Completion Model
// ================================================================

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

    fn create_request(&self, request: CompletionRequest) -> Result<Value, CompletionError> {
        let mut body = serde_json::to_value(&request)
            .map_err(|e| CompletionError::ProviderError(e.to_string()))?;
        body["model"] = serde_json::Value::String(self.model.clone());
        Ok(body)
    }
}

impl completions::CompletionModel for CompletionModel {
    type Response = serde_json::Value;
    type StreamingResponse = crate::streaming::EmptyResponse;

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<serde_json::Value, CompletionError> {
        let body = self.create_request(completion_request)?;

        let response = self
            .client
            .post("/v1/text/chatcompletion_v2")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(CompletionError::ProviderError(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        let t = response.text().await?;
        if t.is_empty() {
            return Err(CompletionError::ProviderError(
                "Empty response from MiniMax API".to_string(),
            ));
        }

        let parsed: CompletionResponse = serde_json::from_str(&t).map_err(|e| {
            CompletionError::ProviderError(format!(
                "Failed to parse MiniMax response: {}. Response was: {}",
                e, t
            ))
        })?;

        if let Some(base) = &parsed.base_resp {
            if base.status_code != 0 {
                return Err(CompletionError::ProviderError(format!(
                    "MiniMax error {}: {}",
                    base.status_code, base.status_msg
                )));
            }
        }

        let raw_response = serde_json::to_value(&parsed).map_err(|e| {
            CompletionError::ProviderError(format!("Failed to serialize response: {}", e))
        })?;

        Ok(raw_response)
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<
        crate::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
        CompletionError,
    > {
        let request_body = self.create_request(request)?;

        // Add streaming parameter for MiniMax
        let mut request_body = request_body;
        request_body["stream"] = serde_json::Value::Bool(true);

        let request_builder = self
            .client
            .post("/v1/text/chatcompletion_v2")
            .json(&request_body);

        let result = send_minimax_streaming_request(request_builder).await?;
        Ok(result)
    }
}

// MiniMax Streaming Implementation
pub async fn send_minimax_streaming_request(
    request_builder: RequestBuilder,
) -> Result<StreamingCompletionResponse<EmptyResponse>, CompletionError> {
    let response = request_builder.send().await?;

    if !response.status().is_success() {
        return Err(CompletionError::ProviderError(format!(
            "{}: {}",
            response.status(),
            response.text().await?
        )));
    }

    // Handle MiniMax SSE streaming chunks
    let inner = Box::pin(stream! {
        let response_text = response.text().await.map_err(|e| {
            CompletionError::ResponseError(format!("Failed to read response: {}", e))
        })?;
        let mut stream: futures::stream::BoxStream<'static, Result<bytes::Bytes, reqwest::Error>> =
            futures::stream::once(async { Ok(bytes::Bytes::from(response_text)) }).boxed();
        let mut final_usage = EmptyResponse;

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    yield Err(CompletionError::HttpError(e));
                    break;
                }
            };

            let text = match String::from_utf8(chunk.to_vec()) {
                Ok(t) => t,
                Err(e) => {
                    yield Err(CompletionError::ResponseError(e.to_string()));
                    break;
                }
            };

            // Process each line in the chunk
            for line in text.lines() {
                let line = line.trim();

                // Skip empty lines
                if line.is_empty() {
                    continue;
                }

                // Handle SSE format: "data: {json}"
                if line.starts_with("data: ") {
                    let json_data = &line[6..]; // Remove "data: " prefix

                    // Handle [DONE] signal
                    if json_data == "[DONE]" {
                        break;
                    }

                    // Try to parse the JSON data
                    match serde_json::from_str::<MiniMaxStreamingChunk>(json_data) {
                        Ok(data) => {
                            // Process each choice in the chunk
                            for choice in data.choices {
                                // Handle delta content (streaming text)
                                if let Some(delta) = choice.delta {
                                    if let Some(content) = delta.content {
                                        yield Ok(RawStreamingChoice::Message(content));
                                    }
                                }

                                // Handle complete message (final response)
                                if let Some(message) = choice.message {
                                    yield Ok(RawStreamingChoice::Message(message.content));
                                }
                            }

                            // Update usage information if available
                            if data.usage.is_some() {
                                // MiniMax doesn't provide detailed usage in streaming,
                                // so we just mark that we have usage info
                                final_usage = EmptyResponse;
                            }
                        }
                        Err(e) => {
                            debug!("Couldn't parse MiniMax streaming chunk: {}", e);
                            continue;
                        }
                    }
                }
            }
        }

        yield Ok(RawStreamingChoice::FinalResponse(final_usage))
    });

    Ok(StreamingCompletionResponse::new(inner))
}
