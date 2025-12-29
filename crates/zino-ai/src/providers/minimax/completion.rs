use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::Client;
use crate::completions::{
    self, CompletionError, CompletionRequest, CompletionResponse, Message, RawStreamingChoice,
};
use async_stream::stream;
use futures::StreamExt;
use reqwest::RequestBuilder;
use tracing::debug;

// ================================================================
// MiniMax Completion API
// Docs (summary from user): https://api.minimaxi.com/v1/text/chatcompletion_v2
// ================================================================

// MiniMax Streaming Response Structures

/// Delta information for streaming responses from MiniMax.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingDelta {
    /// Content delta for streaming responses.
    #[serde(default)]
    content: Option<String>,
    /// Role delta for streaming responses.
    #[serde(default)]
    role: Option<String>,
}

/// Choice information for streaming responses from MiniMax.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingChoice {
    /// Index of the choice.
    index: usize,
    /// Delta information for this choice.
    #[serde(default)]
    delta: Option<MiniMaxStreamingDelta>,
    /// Complete message for this choice.
    #[serde(default)]
    message: Option<MiniMaxStreamingMessage>,
    /// Reason for completion.
    #[serde(default)]
    finish_reason: Option<String>,
}

/// Complete message for streaming responses from MiniMax.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingMessage {
    /// Message content.
    content: String,
    /// Message role.
    role: String,
}

/// Usage information for streaming responses from MiniMax.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingUsage {
    /// Total number of tokens used.
    total_tokens: u64,
}

/// Streaming chunk for MiniMax responses.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiniMaxStreamingChunk {
    /// Unique identifier for the chunk.
    id: String,
    /// Available choices for this chunk.
    choices: Vec<MiniMaxStreamingChoice>,
    /// Creation timestamp.
    created: u64,
    /// Model name used.
    model: String,
    /// Object type.
    object: String,
    /// Usage information for this chunk.
    #[serde(default)]
    usage: Option<MiniMaxStreamingUsage>,
}

/// Model identifier for MiniMax M1.
pub const MINIMAX_M1: &str = "MiniMax-M1";
/// Model identifier for MiniMax Text-01.
pub const MINIMAX_TEXT_01: &str = "MiniMax-Text-01";

// ================================================================
// Request/Response Models (MiniMax specific)
// ================================================================

/// Base response structure for MiniMax API.
#[derive(Debug, Deserialize, Serialize)]
pub struct BaseResp {
    /// HTTP status code.
    pub status_code: i32,
    /// Status message.
    #[serde(default)]
    pub status_msg: String,
}

/// Choice information for MiniMax completions.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Choice {
    /// Index of this choice.
    pub index: usize,
    /// Message content for this choice.
    pub message: Message,
    /// Reason for completion.
    #[serde(default)]
    pub finish_reason: String,
}

/// Usage information for MiniMax completions.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Usage {
    /// Total number of tokens used.
    pub total_tokens: u64,
}

/// Streaming completion response for MiniMax.
#[derive(Debug, Deserialize, Clone)]
pub struct StreamingCompletionResponse {
    /// Usage information for the completion.
    pub usage: Option<Usage>,
}

// ================================================================
// Completion Model
// ================================================================

/// MiniMax completion model implementation.
#[derive(Clone)]
pub struct CompletionModel {
    /// HTTP client for making requests.
    pub(crate) client: Client,
    /// Model name to use for completions.
    pub model: String,
}

impl CompletionModel {
    /// Creates a new MiniMax completion model.
    ///
    /// # Arguments
    /// * `client` - The HTTP client for making requests.
    /// * `model` - The model name to use for completions.
    ///
    /// # Returns
    /// A new `CompletionModel` instance.
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Creates a request body for the MiniMax API.
    ///
    /// # Arguments
    /// * `request` - The completion request to convert.
    ///
    /// # Returns
    /// A JSON value representing the request body, or an error if conversion fails.
    fn create_request(&self, request: CompletionRequest) -> Result<Value, CompletionError> {
        let mut body = serde_json::to_value(&request)
            .map_err(|e| CompletionError::ProviderError(e.to_string()))?;
        body["model"] = serde_json::Value::String(self.model.clone());
        Ok(body)
    }
}

impl completions::CompletionModel for CompletionModel {
    type Response = serde_json::Value;

    type StreamingResponse = StreamingCompletionResponse;

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<CompletionResponse<Self::Response>, CompletionError> {
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

        // parse response
        let result: serde_json::Value = response.json().await?;
        Ok(CompletionResponse {
            raw_response: result,
        })
    }
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<completions::StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
    {
        CompletionModel::stream(self, request).await
    }
}

impl CompletionModel {
    /// Handles streaming completion requests for MiniMax.
    ///
    /// # Arguments
    /// * `request` - The completion request to stream.
    ///
    /// # Returns
    /// A streaming completion response, or an error if the request fails.
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<
        completions::streaming::StreamingCompletionResponse<StreamingCompletionResponse>,
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

        send_compatible_streaming_request(request_builder).await
    }
}
// MiniMax Streaming Implementation
/// Sends a streaming request to MiniMax and processes the response.
///
/// # Arguments
/// * `request_builder` - The prepared request builder.
///
/// # Returns
/// A streaming completion response with usage information, or an error if the request fails.
pub async fn send_compatible_streaming_request(
    request_builder: RequestBuilder,
) -> Result<
    completions::streaming::StreamingCompletionResponse<StreamingCompletionResponse>,
    CompletionError,
> {
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
        let mut stream = response.bytes_stream();
        let mut partial_data = None;
        let mut final_usage: Option<Usage> = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    yield Err(CompletionError::from(e));
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

            for line in text.lines() {
                let mut line = line.to_string();

                // If there was a remaining part, concat with current line
                if partial_data.is_some() {
                    line = format!("{}{}", partial_data.unwrap(), line);
                    partial_data = None;
                }

                // Handle SSE data lines
                if line.starts_with("data:") {
                    let data = line.strip_prefix("data:").unwrap_or("").trim_start();

                    // Check for [DONE] marker
                    if data == "[DONE]" {
                        debug!("Received [DONE] marker, ending stream");
                        break;
                    }

                    // Handle partial JSON data
                    if !data.ends_with("}") && !data.ends_with("]") {
                        partial_data = Some(data.to_string());
                        continue;
                    }

                    // Try to parse as JSON
                    if data.is_empty() {
                        continue;
                    }

                    // Parse the JSON data
                    let json_data: serde_json::Value = match serde_json::from_str(data) {
                        Ok(json) => json,
                        Err(e) => {
                            debug!("Failed to parse JSON data: {:?}, data: {}", e, data);
                            continue;
                        }
                    };

                    debug!("Received chunk: {}", serde_json::to_string(&json_data).unwrap_or_default());

                    // Extract content from the response
                    if let Some(choices) = json_data.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(delta) = choice.get("delta")
                                && let Some(content) = delta.get("content").and_then(|c| c.as_str())
                                && !content.is_empty()
                            {
                                yield Ok(RawStreamingChoice::Message(content.to_string()));
                            }
                        }
                    }

                    // Handle usage information (only in the last chunk when include_usage=true)
                    if let Some(usage) = json_data.get("usage")
                        && !usage.is_null()
                        && let Ok(usage_data) = serde_json::from_value::<Usage>(usage.clone())
                    {
                        final_usage = Some(usage_data);
                    }
                }
            }
        }

        // Yield final response with usage information
        yield Ok(RawStreamingChoice::FinalResponse(StreamingCompletionResponse {
            usage: final_usage,
        }));
    });

    Ok(completions::streaming::StreamingCompletionResponse::stream(
        inner,
    ))
}
