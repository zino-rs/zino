use crate::completions::CompletionError;
use crate::providers::qwen::completion::Usage;
use crate::streaming::{
    RawStreamingChoice, StreamingCompletionResponse as GlobalStreamingCompletionResponse,
};
use async_stream::stream;
use futures::StreamExt;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

// ================================================================
// Qwen Streaming API Implementation
// ================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamingFunction {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamingToolCall {
    pub index: usize,
    pub id: Option<String>,
    pub function: StreamingFunction,
}

#[derive(Deserialize, Debug)]
struct StreamingDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<StreamingToolCall>,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Deserialize, Debug)]
struct StreamingChoice {
    delta: StreamingDelta,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    index: usize,
}

#[derive(Deserialize, Debug)]
struct StreamingCompletionChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<StreamingChoice>,
    usage: Option<Usage>,
}

#[derive(Clone)]
pub struct QwenStreamingCompletionResponse {
    pub usage: Usage,
}

pub async fn send_compatible_streaming_request(
    request_builder: RequestBuilder,
) -> Result<GlobalStreamingCompletionResponse<QwenStreamingCompletionResponse>, CompletionError> {
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
        let response_text = response.text().await.map_err(|e| {
            CompletionError::ResponseError(format!("Failed to read response: {}", e))
        })?;
        let mut stream: futures::stream::BoxStream<'static, Result<bytes::Bytes, reqwest::Error>> =
            futures::stream::once(async { Ok(bytes::Bytes::from(response_text)) }).boxed();
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
                if line.starts_with("data: ") {
                    let json_data = &line[6..]; // Remove "data: " prefix

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

        yield Ok(RawStreamingChoice::FinalResponse(QwenStreamingCompletionResponse {
            usage: final_usage,
        }))
    });

    Ok(GlobalStreamingCompletionResponse::new(inner))
}
