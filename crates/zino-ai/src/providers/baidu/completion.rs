//pub mod streaming;
use super::client::Client;
use crate::completions::{self, CompletionError, CompletionRequest, CompletionResponse};
use async_stream::stream;
use futures::StreamExt;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

/// Streaming completion chunk for Baidu responses.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingCompletionChunk {
    /// Available choices for this chunk.
    pub choices: Vec<StreamingChoice>,
}

/// Choice information for streaming responses from Baidu.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingChoice {
    /// Delta information for this choice.
    pub delta: StreamingDelta,
}

/// Delta information for streaming responses from Baidu.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingDelta {
    /// Content delta for streaming responses.
    pub content: Option<String>,
    /// Tool calls delta for streaming responses.
    pub tool_calls: Vec<StreamingToolCall>,
    /// Role delta for streaming responses.
    pub role: Option<String>,
}

/// Tool call information for streaming responses from Baidu.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingToolCall {
    /// Unique identifier for the tool call.
    pub id: Option<String>,
    /// Type of the tool call.
    pub r#type: String,
    /// Function information for this tool call.
    pub function: StreamingFunction,
}

/// Function information for streaming tool calls from Baidu.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingFunction {
    /// Name of the function.
    #[serde(default)]
    name: Option<String>,
    /// Arguments for the function call.
    #[serde(default)]
    arguments: String,
}

/// Usage information for Baidu completions.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Usage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: u64,
    /// Detailed information about prompt tokens.
    pub prompt_tokens_details: Option<UsageDetails>,
    /// Number of tokens in the completion.
    pub completion_tokens: u64,
    /// Total number of tokens used.
    pub total_tokens: u64,
}

/// Detailed usage information for Baidu completions.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageDetails {
    /// Number of search tokens used.
    pub search_tokens: Option<u64>,
    /// Number of cached tokens used.
    pub cached_tokens: u64,
}

/// Streaming completion response for Baidu.
#[derive(Debug, Deserialize, Clone)]
pub struct StreamingCompletionResponse {
    /// Usage information for the completion.
    pub usage: Option<Usage>,
}

/// Baidu completion model implementation.
#[derive(Clone)]
pub struct CompletionModel {
    /// Model name to use for completions.
    model: String,
    /// HTTP client for making requests.
    client: Client,
}

impl CompletionModel {
    /// Creates a new Baidu completion model.
    ///
    /// # Arguments
    /// * `model` - The model name to use for completions.
    /// * `client` - The HTTP client for making requests.
    ///
    /// # Returns
    /// A new `CompletionModel` instance.
    pub fn new(model: &str, client: Client) -> Self {
        Self {
            model: model.to_string(),
            client,
        }
    }

    /// Creates a request body for the Baidu API.
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
        let mut request_body = serde_json::to_value(&completion_request)?;
        request_body["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_body)
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
        debug!("{}", "=".repeat(50));
        debug!("message to send: {}", request["messages"]);
        debug!("{}", "=".repeat(50));
        // send post request to baidu qianfan api
        let response = self
            .client
            .post("v2/chat/completions")
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
    /// Handles streaming completion requests for Baidu.
    ///
    /// # Arguments
    /// * `completion_request` - The completion request to stream.
    ///
    /// # Returns
    /// A streaming completion response with usage information, or an error if the request fails.
    pub(crate) async fn stream(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<
        completions::streaming::StreamingCompletionResponse<StreamingCompletionResponse>,
        CompletionError,
    > {
        let mut request = self.create_completion_request(completion_request)?;
        // set stream parameter
        request["stream"] = serde_json::Value::Bool(true);

        // Add stream_options to include usage information
        request["stream_options"] = serde_json::json!({
            "include_usage": true
        });

        // send post request to baidu qianfan api
        let response = self.client.post("v2/chat/completions").json(&request);

        send_compatible_streaming_request(response).await
    }
}
/// Sends a streaming request to Baidu and processes the response.
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

    // Handle Baidu SSE chunks
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
                            if let Some(delta) = choice.get("delta") {
                                if let Some(content) = delta.get("content").and_then(|c| c.as_str())
                                    && !content.is_empty()
                                {
                                    yield Ok(completions::streaming::RawStreamingChoice::Message(content.to_string()));
                                }

                                // Handle tool calls if present
                                if let Some(tool_calls) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                                    for tool_call in tool_calls {
                                        if let Some(function) = tool_call.get("function") {
                                            let name = function.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                            let arguments = function.get("arguments").and_then(|a| a.as_str()).unwrap_or("");

                                            if !name.is_empty() && !arguments.is_empty() {
                                                let id = tool_call.get("id").and_then(|i| i.as_str()).unwrap_or("");
                                                let arguments_json: serde_json::Value = match serde_json::from_str(arguments) {
                                                    Ok(args) => args,
                                                    Err(_) => serde_json::Value::String(arguments.to_string()),
                                                };

                                                yield Ok(completions::streaming::RawStreamingChoice::ToolCall {
                                                    id: id.to_string(),
                                                    name: name.to_string(),
                                                    arguments: arguments_json,
                                                    call_id: None,
                                                });
                                            }
                                        }
                                    }
                                }
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
        yield Ok(completions::streaming::RawStreamingChoice::FinalResponse(StreamingCompletionResponse {
            usage: final_usage,
        }));
    });

    Ok(completions::streaming::StreamingCompletionResponse::stream(
        inner,
    ))
}
