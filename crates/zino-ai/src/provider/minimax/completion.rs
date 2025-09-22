use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::client::Client;
use crate::OneOrMany;
use crate::completions::{self,CompletionError, CompletionRequest};
use crate::json_utils;
use crate::message;
use crate::streaming;

// ================================================================
// MiniMax Completion API
// Docs (summary from user): https://api.minimaxi.com/v1/text/chatcompletion_v2
// ================================================================

pub const MINIMAX_M1: &str = "MiniMax-M1";
pub const MINIMAX_TEXT_01: &str = "MiniMax-Text-01";

// ================================================================
// Request/Response Models (MiniMax specific)
// ================================================================

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct BaseResp {
    pub status_code: i32,
    #[serde(default)]
    pub status_msg: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    #[serde(default)]
    pub finish_reason: String,
}

#[derive(Clone, Debug, Deserialize)]
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

impl completion::CompletionModel for CompletionModel {
    type Response = CompletionResponse;
    type StreamingResponse =
        crate::providers::qwen::completion::streaming::StreamingCompletionResponse;

    #[cfg_attr(feature = "worker", worker::send)]
    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<completions::CompletionResponse<CompletionResponse>, CompletionError> {
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

        // Map to rig::completion::CompletionResponse
        let choice = parsed.choices.first().ok_or_else(|| {
            CompletionError::ResponseError("Response contained no choices".to_string())
        })?;

        let mut content_items: Vec<completion::message::AssistantContent> = vec![];
        match &choice.message {
            Message::Assistant {
                content,
                tool_calls,
                ..
            } => {
                if let Some(text) = content.as_ref() {
                    if !text.is_empty() {
                        content_items.push(completion::message::AssistantContent::text(text));
                    }
                }

                for call in tool_calls {
                    content_items.push(completion::message::AssistantContent::tool_call(
                        &call.id,
                        &call.function.name,
                        call.function.arguments.clone(),
                    ));
                }
            }
            Message::User { .. } | Message::System { .. } | Message::ToolResult { .. } => {
                return Err(CompletionError::ResponseError(
                    "Response did not contain an assistant message".into(),
                ));
            }
        }

        let choice = OneOrMany::many(content_items).map_err(|_| {
            CompletionError::ResponseError(
                "Response contained no message or tool call (empty)".to_owned(),
            )
        })?;

        let usage = parsed.usage.clone().map(|u| u.into()).unwrap_or_default();

        Ok(completion::CompletionResponse {
            choice,
            usage,
            raw_response: parsed,
        })
    }

    #[cfg_attr(feature = "worker", worker::send)]
    async fn stream(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<streaming::StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
    {
        // Build request with stream enabled and include usage if supported
        let mut body = self.create_request(completion_request)?;

        let builder = self.client.post("/v1/text/chatcompletion_v2").json(&body);

        // Reuse OpenAI-compatible streaming handler used for Qwen
        crate::providers::qwen::completion::streaming::send_compatible_streaming_request(builder)
            .await
    }
}
