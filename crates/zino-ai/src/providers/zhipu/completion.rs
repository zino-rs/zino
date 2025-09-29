use super::client::Client;
use crate::completions::streaming::RawStreamingChoice;
use crate::completions::{self, CompletionError, CompletionRequest, CompletionResponse};
use async_stream::stream;
use futures::StreamExt;
use reqwest::RequestBuilder;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

/// A streaming completion chunk from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingCompletionChunk {
    /// Unique identifier for the completion.
    pub id: Option<String>,
    /// Unix timestamp when the completion was created.
    pub created: Option<u64>,
    /// The model used for the completion.
    pub model: Option<String>,
    /// List of completion choices.
    pub choices: Vec<StreamingChoice>,
    /// Token usage information.
    pub usage: Option<Usage>,
}

/// A streaming completion choice from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingChoice {
    /// Index of the choice.
    pub index: Option<usize>,
    /// Reason why the completion finished.
    pub finish_reason: Option<String>,
    /// Delta content for this choice.
    pub delta: StreamingDelta,
}

/// Delta content for streaming completions from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingDelta {
    /// Text content of the completion.
    pub content: Option<String>,
    /// Reasoning content from Zhipu AI's thinking process.
    pub reasoning_content: Option<String>,
    /// Role of the message (e.g., "assistant").
    pub role: Option<String>,
    /// Tool calls made during the completion.
    pub tool_calls: Option<Vec<StreamingToolCall>>,
}

/// A streaming tool call from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingToolCall {
    /// Index of the tool call.
    pub index: usize,
    /// Unique identifier for the tool call.
    pub id: Option<String>,
    /// Function information for this tool call.
    pub function: StreamingFunction,
}

/// A streaming function call from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingFunction {
    /// Name of the function to call.
    pub name: Option<String>,
    /// Arguments for the function call.
    pub arguments: String,
}

/// Token usage information from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Usage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,
    /// Number of tokens in the completion.
    pub completion_tokens: usize,
    /// Detailed information about prompt tokens.
    pub prompt_tokens_details: PromptTokensDetails,
    /// Total number of tokens used.
    pub total_tokens: usize,
}

/// Detailed information about prompt tokens from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptTokensDetails {
    /// Number of cached tokens.
    pub cached_tokens: usize,
}

/// Final streaming completion response from Zhipu AI API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingCompletionResponse {
    /// Token usage information.
    pub usage: Option<Usage>,
}

/// Zhipu AI completion model for making API requests.
#[derive(Debug, Clone)]
pub struct CompletionModel {
    /// The model name to use for completions.
    pub model: String,
    /// HTTP client for making requests.
    pub(crate) client: Client,
}

impl CompletionModel {
    /// Creates a new Zhipu AI completion model.
    ///
    /// # Arguments
    /// * `name` - The model name to use for completions.
    /// * `client` - The HTTP client for making requests.
    ///
    /// # Returns
    /// A new `CompletionModel` instance.
    pub fn new(name: &str, client: Client) -> Self {
        Self {
            model: name.to_string(),
            client,
        }
    }

    pub(crate) fn create_completion_request(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<Value, CompletionError> {
        // Build up the order of messages (context, chat_history)
        let mut request_json = serde_json::to_value(&completion_request)?;
        request_json["model"] = serde_json::Value::String(self.model.clone());

        // Debug: print the request before sending
        debug!(
            "Zhipu request JSON: {}",
            serde_json::to_string_pretty(&request_json).unwrap_or_default()
        );

        Ok(request_json)
    }
}

impl completions::CompletionModel for CompletionModel {
    type Response = serde_json::Value;

    type StreamingResponse = StreamingCompletionResponse;

    //none streaming completion request

    async fn completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse<Self::Response>, CompletionError> {
        //create a json format request from CompletionRequest object
        let request_value = self.create_completion_request(request)?;
        //send http request
        let response = self
            .client
            .post("/api/paas/v4/chat/completions")
            .json(&request_value)
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
    /// Handles streaming completion requests for Zhipu.
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

        // Add streaming parameter for Zhipu API
        request_body["stream"] = serde_json::Value::Bool(true);

        // Debug: print request body
        debug!(
            "Zhipu request body: {}",
            serde_json::to_string_pretty(&request_body).unwrap_or_default()
        );

        let request_builder = self
            .client
            .post("/api/paas/v4/chat/completions")
            .json(&request_body);

        send_compatible_streaming_request(request_builder).await
    }
}

/// Sends a streaming completion request to Zhipu AI API.
///
/// # Arguments
/// * `request_builder` - The configured request builder.
///
/// # Returns
/// A streaming completion response or an error if the request fails.
pub async fn send_compatible_streaming_request(
    request_builder: RequestBuilder,
) -> Result<
    completions::streaming::StreamingCompletionResponse<StreamingCompletionResponse>,
    CompletionError,
> {
    let response = request_builder.send().await?;

    debug!("Zhipu response status: {}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        debug!("Zhipu error response: {}", error_text);
        return Err(CompletionError::ProviderError(format!(
            "{}: {}",
            status, error_text
        )));
    }

    // Handle Qwen SSE streaming chunks
    let inner = Box::pin(stream! {
        let mut stream = response.bytes_stream();
        let mut final_usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            prompt_tokens_details: PromptTokensDetails {
                cached_tokens: 0,
            },
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
            debug!("Zhipu text: {}", text);
            // Process each line in the chunk
            for line in text.lines() {
                let line = line.trim();

                // Skip empty lines
                if line.is_empty() {
                    continue;
                }

                // Debug: print raw line data
                debug!("Raw line: {}", line);

                // Handle SSE format: "data: {json}"
                if let Some(json_data) = line.strip_prefix("data: ") {
                    // Remove "data: " prefix

                    // Handle [DONE] signal
                    if json_data == "[DONE]" {
                        break;
                    }

                    // Debug: print JSON data
                    debug!("JSON data: {}", json_data);

                    // Try to parse the JSON data
                    match serde_json::from_str::<StreamingCompletionChunk>(json_data) {
                        Ok(data) => {
                            debug!("Parsed chunk successfully: {:?}", data);
                            // Process each choice in the chunk
                            for choice in data.choices {
                                let delta = &choice.delta;

                                // Handle tool calls
                                if let Some(tool_calls) = &delta.tool_calls {
                                    if !tool_calls.is_empty() {
                                        for tool_call in tool_calls {
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
                                }

                                // Handle text content - support both content and reasoning_content
                                if let Some(content) = &delta.content {
                                    yield Ok(RawStreamingChoice::Message(content.clone()));
                                }

                                // Handle reasoning content
                                if let Some(reasoning_content) = &delta.reasoning_content {
                                    yield Ok(RawStreamingChoice::Message(reasoning_content.clone()));
                                }
                            }

                            // Update usage information - accumulate usage data
                            if let Some(usage) = data.usage {
                                // Accumulate usage information instead of replacing
                                final_usage.prompt_tokens = usage.prompt_tokens;
                                final_usage.completion_tokens = usage.completion_tokens;
                                final_usage.total_tokens = usage.total_tokens;
                                final_usage.prompt_tokens_details.cached_tokens = usage.prompt_tokens_details.cached_tokens;
                            }
                        }
                        Err(e) => {
                            debug!("Couldn't parse streaming chunk: {}", e);
                            debug!("Failed JSON data: {}", json_data);
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
