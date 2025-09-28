// ================================================================
// Qwen Completion API
// ================================================================
use super::{ApiErrorResponse, Client};
use crate::completions::streaming::RawStreamingChoice;
use crate::completions::{self, CompletionError, CompletionRequest, CompletionResponse, Message};
use async_stream::stream;
use futures::StreamExt;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use tracing::debug;

impl From<ApiErrorResponse> for CompletionError {
    fn from(err: ApiErrorResponse) -> Self {
        CompletionError::ProviderError(err.message)
    }
}

/// Function information for streaming tool calls.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamingFunction {
    /// Name of the function.
    #[serde(default)]
    name: Option<String>,
    /// Arguments for the function call.
    #[serde(default)]
    arguments: String,
}

/// Tool call information for streaming responses.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamingToolCall {
    /// Index of the tool call.
    pub index: usize,
    /// Unique identifier for the tool call.
    pub id: Option<String>,
    /// Function information for this tool call.
    pub function: StreamingFunction,
}

/// Delta information for streaming responses.
#[derive(Deserialize, Debug)]
struct StreamingDelta {
    /// Content delta for streaming responses.
    #[serde(default)]
    content: Option<String>,
    /// Tool calls delta for streaming responses.
    #[serde(default)]
    tool_calls: Vec<StreamingToolCall>,
}

/// Choice information for streaming responses.
#[derive(Deserialize, Debug)]
struct StreamingChoice {
    /// Delta information for this choice.
    delta: StreamingDelta,
}

/// Streaming completion chunk for Qwen responses.
#[derive(Deserialize, Debug)]
struct StreamingCompletionChunk {
    /// Available choices for this chunk.
    choices: Vec<StreamingChoice>,
    /// Usage information for this chunk.
    usage: Option<Usage>,
}

/// Choice information for Qwen completions.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Choice {
    /// Index of this choice.
    pub index: usize,
    /// Message content for this choice.
    pub message: Message,
    /// Log probabilities for this choice.
    pub logprobs: Option<serde_json::Value>,
    /// Reason for completion.
    pub finish_reason: String,
}

/// Usage information for Qwen completions.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Usage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,
    /// Total number of tokens used.
    pub total_tokens: usize,
}

/// Streaming completion response for Qwen.
#[derive(Debug, Deserialize, Clone)]
pub struct StreamingCompletionResponse {
    /// Usage information for the completion.
    pub usage: Option<Usage>,
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

/// Qwen completion model implementation.
#[derive(Clone)]
pub struct CompletionModel {
    /// HTTP client for making requests.
    pub(crate) client: Client,
    /// Model name to use for completions.
    pub model: String,
}

impl CompletionModel {
    /// Creates a new Qwen completion model.
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

    /// Creates a request body for the Qwen API.
    ///
    /// # Arguments
    /// * `completion_request` - The completion request to convert.
    ///
    /// # Returns
    /// A JSON value representing the request body, or an error if conversion fails.
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

    type StreamingResponse = StreamingCompletionResponse;

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<CompletionResponse<Self::Response>, CompletionError> {
        let request = self.create_completion_request(completion_request)?;

        let response = self
            .client
            .post("/compatible-mode/v1/chat/completions")
            .json(&request)
            .send()
            .await?;

        // check http status code
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(CompletionError::ProviderError(format!(
                "HTTP {}: {}",
                status, error_text
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
    /// Handles streaming completion requests for Qwen.
    ///
    /// # Arguments
    /// * `request` - The completion request to stream.
    ///
    /// # Returns
    /// A streaming completion response with usage information, or an error if the request fails.
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<
        completions::streaming::StreamingCompletionResponse<StreamingCompletionResponse>,
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

        send_compatible_streaming_request(request_builder).await
    }
}

/// Sends a streaming request to Qwen and processes the response.
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

    // Handle Qwen SSE streaming chunks
    let inner = Box::pin(stream! {
        let mut stream = response.bytes_stream();
        let mut final_usage = Usage {
            prompt_tokens: 0,
            total_tokens: 0
        };

        let mut calls: HashMap<usize, (String, String, String)> = HashMap::new();

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
                if let Some(json_data) = line.strip_prefix("data: ") {
                    // Remove "data: " prefix

                    // Handle [DONE] signal
                    if json_data == "[DONE]" {
                        break;
                    }

                    // Try to parse the JSON data
                    match serde_json::from_str::<StreamingCompletionChunk>(json_data) {
                        Ok(data) => {
                            // Process each choice in the chunk
                            for choice in data.choices {
                                let delta = &choice.delta;

                                // Handle tool calls
                                if !delta.tool_calls.is_empty() {
                                    for tool_call in &delta.tool_calls {
                                        let function = tool_call.function.clone();

                                        // Start of tool call
                                        if function.name.is_some() && function.arguments.is_empty() {
                                            let id = tool_call.id.clone().unwrap_or("".to_string());
                                            calls.insert(tool_call.index, (id, function.name.clone().unwrap(), "".to_string()));
                                        }
                                        // Part of tool call arguments
                                        else if function.name.clone().is_none_or(|s| s.is_empty()) && !function.arguments.is_empty() {
                                            if let Some((id, name, arguments)) = calls.get(&tool_call.index) {
                                                let new_arguments = &tool_call.function.arguments;
                                                let arguments = format!("{arguments}{new_arguments}");
                                                calls.insert(tool_call.index, (id.clone(), name.clone(), arguments));
                                            }
                                        }
                                        // Complete tool call
                                        else {
                                            let id = tool_call.id.clone().unwrap_or("".to_string());
                                            let name = function.name.expect("function name should be present for complete tool call");
                                            let arguments = function.arguments;

                                            match serde_json::from_str(&arguments) {
                                                Ok(arguments) => {
                                                    yield Ok(RawStreamingChoice::ToolCall {
                                                        id,
                                                        name,
                                                        arguments,
                                                        call_id: None
                                                    });
                                                }
                                                Err(e) => {
                                                    debug!("Couldn't serialize '{}' as a json value: {}", arguments, e);
                                                }
                                            }
                                        }
                                    }
                                }

                                // Handle text content
                                if let Some(content) = &delta.content {
                                    yield Ok(RawStreamingChoice::Message(content.clone()));
                                }
                            }

                            // Update usage information
                            if let Some(usage) = data.usage {
                                final_usage = usage.clone();
                            }
                        }
                        Err(e) => {
                            debug!("Couldn't parse streaming chunk: {}", e);
                            continue;
                        }
                    }
                }
            }
        }

        // Yield any remaining tool calls
        for (_, (id, name, arguments)) in calls {
            match serde_json::from_str(&arguments) {
                Ok(arguments) => {
                    yield Ok(RawStreamingChoice::ToolCall {
                        id,
                        name,
                        arguments,
                        call_id: None
                    });
                }
                Err(e) => {
                    debug!("Couldn't serialize remaining tool call arguments: {}", e);
                }
            }
        }

        yield Ok(RawStreamingChoice::FinalResponse(StreamingCompletionResponse {
            usage: Some(final_usage),
        }))
    });

    Ok(completions::streaming::StreamingCompletionResponse::stream(
        inner,
    ))
}
