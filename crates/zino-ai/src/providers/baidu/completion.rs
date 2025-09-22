//pub mod streaming;
use super::client::Client;
use crate::completions::{Message, Role};
use crate::tool::ToolCall;
use bytes::Bytes;
use futures::StreamExt;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

// Errors
#[derive(Debug, Error)]
pub enum CompletionError {
    /// Http error (e.g.: connection error, timeout, etc.)
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Json error (e.g.: serialization, deserialization)
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error building the completion request
    #[error("RequestError: {0}")]
    RequestError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Error parsing the completion response
    #[error("ResponseError: {0}")]
    ResponseError(String),

    /// Error returned by the completion model provider
    #[error("ProviderError: {0}")]
    ProviderError(String),

    /// Custom error for general use
    #[error("Custom: {0}")]
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CompletionRequest {
    //pub model: String,
    pub messages: Vec<Message>,
    pub additional_params: Option<serde_json::Value>,
}
impl CompletionRequest {
    pub fn new(message: Vec<Message>) -> Self {
        Self {
            messages: message,
            ..Self::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResult {
    pub id: String,
    pub object: String,
    pub created: usize,
    pub model: String,
    pub choices: Option<Vec<NonStreamChoice>>,
    pub usage: Option<Usage>,
    pub search_result: Option<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonStreamChoice {
    pub index: Option<usize>,
    pub message: Option<ChoiceMessage>,
    pub finish_reason: Option<String>,
    pub flag: Option<usize>,
    pub ban_round: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChoiceMessage {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamCompletionResult {
    pub id: String,
    pub object: String,
    pub created: usize,
    pub model: String,
    pub choices: Option<Vec<StreamChunk>>,
    pub usage: Option<Usage>,
    pub search_result: Option<SearchResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StreamChunk {
    pub index: usize,
    pub delta: Option<Delta>,
    pub delta_flag: Option<String>,
    pub finish_reason: Option<String>,
    pub flag: usize,
    pub ban_round: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<Role>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<usize>,
    pub prompt_token_details: Option<TokenDetail>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetail {
    pub search_tokens: usize,
    pub cached_tokens: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub index: Option<usize>,
    pub url: Option<String>,
    pub title: Option<String>,
}
pub struct CompletionModel {
    pub model: String,
    pub client: Client,
}

impl CompletionModel {
    pub fn new(model: &str, client: Client) -> Self {
        Self {
            model: model.to_string(),
            client: client,
        }
    }
}

impl CompletionModel {
    pub(crate) fn create_completion_request(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<Value, CompletionError> {
        let mut request_body = serde_json::to_value(&completion_request)?;
        request_body["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_body)
    }
    pub async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<CompletionResult, CompletionError> {
        let request = self.create_completion_request(completion_request)?;
        println!("{}", "=".repeat(50));
        println!("message to send: {}", request["messages"]);
        println!("{}", "=".repeat(50));
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
        let result: CompletionResult = response.json().await?;
        Ok(result)
    }

    /// * `Result<impl futures::Stream<Item = Result<CompletionResult, CompletionError>>, CompletionError>` - parsed stream response
    pub async fn completion_stream(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<
        impl futures::Stream<Item = Result<StreamCompletionResult, CompletionError>>,
        CompletionError,
    > {
        let mut request = self.create_completion_request(completion_request)?;
        // set stream parameter
        request["stream"] = serde_json::Value::Bool(true);

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
            let err_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(CompletionError::ResponseError(format!(
                "HTTP {}: {}",
                status, err_text
            )));
        }

        // get response stream and parse
        // Since bytes_stream requires 'stream' feature, we'll use a different approach
        // For now, let's read the entire response and process it
        let response_text = response.text().await?;
        let stream = futures::stream::once(async { Ok(bytes::Bytes::from(response_text)) }).boxed();

        let parsed_stream = stream.map(|chunk_result| {
            chunk_result
                .map_err(|e| CompletionError::HttpError(e))
                .and_then(|chunk: Bytes| {
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    Ok(chunk_str.to_string())
                })
        });

        // use async_stream to handle buffer and parse
        let s = async_stream::stream! {
            let mut buffer = String::new();
            let mut stream = parsed_stream;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk_str) => {
                        buffer.push_str(&chunk_str);

                        // extract and parse by line
                        loop {
                            if let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].trim().to_string();
                                buffer.drain(..=pos);
                                if line.is_empty() { continue; }

                                // remove SSE prefix
                                let json_str = line.strip_prefix("data:").map(|s| s.trim()).unwrap_or(line.as_str());

                                // check if it is the end of stream
                                if json_str == "[DONE]" {
                                    println!("Stream completed with [DONE]");
                                    break;
                                }

                                // parse to CompletionResult
                                match serde_json::from_str::<StreamCompletionResult>(json_str) {
                                    Ok(result) => {
                                        yield Ok(result);
                                    }
                                    Err(e) => {
                                        println!("Failed to parse JSON: {}", e);
                                        println!("JSON string: {}", json_str);
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                        break;
                    }
                }
            }

            // handle tail residual data
            let tail = buffer.trim();
            if !tail.is_empty() {
                let json_str = tail.strip_prefix("data:").map(|s| s.trim()).unwrap_or(tail);
                //check if it is the end of stream
                if json_str == "[DONE]" {
                    println!("Stream completed with [DONE] (tail)");
                } else if let Ok(result) = serde_json::from_str::<StreamCompletionResult>(json_str) {
                    yield Ok(result);
                }
            }
        };

        Ok(s)
    }

    pub async fn completion_stream_bytes(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<impl futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>>, CompletionError>
    {
        let mut request = self.create_completion_request(completion_request)?;
        request["stream"] = serde_json::Value::Bool(true);

        let response = self
            .client
            .post("v2/chat/completions")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(CompletionError::ResponseError(format!(
                "HTTP {}: {}",
                status, err_text
            )));
        }

        let response_text = response.text().await?;
        Ok(futures::stream::once(async { Ok(bytes::Bytes::from(response_text)) }).boxed())
    }
}
