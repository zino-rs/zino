use super::Client;
use crate::audio_generation::AudioGenerationError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// MiniMax T2A Async endpoint
const T2A_ASYNC_PATH: &str = "/v1/t2a_async_v2";

#[derive(Clone)]
pub struct AsyncT2AModel {
    pub(crate) client: Client,
    pub model: String,
}

impl AsyncT2AModel {
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Create a long-text T2A task. `group_id` is required by MiniMax and must be appended in URL.
    /// The `payload` should contain fields from docs like `text` or `text_file_id`, `voice_setting`,
    /// `audio_setting`, `language_boost`, etc. The `model` field will be injected automatically if missing.
    pub async fn create_task(
        &self,
        mut payload: Value,
        group_id: &str,
    ) -> Result<CreateTaskResponse, AudioGenerationError> {
        if payload.get("model").is_none() {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("model".to_string(), Value::String(self.model.clone()));
            }
        }

        let url = format!("{}?GroupId={}", T2A_ASYNC_PATH, group_id);
        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(AudioGenerationError::ProviderError(format!(
                "{}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let text = response.text().await?;
        if text.is_empty() {
            return Err(AudioGenerationError::ProviderError(
                "Empty response from MiniMax T2A Async".to_string(),
            ));
        }

        let parsed: CreateTaskResponse = serde_json::from_str(&text).map_err(|e| {
            AudioGenerationError::ProviderError(format!(
                "Failed to parse T2A Async response: {}. Response was: {}",
                e, text
            ))
        })?;

        if parsed.base_resp.status_code != 0 {
            return Err(AudioGenerationError::ProviderError(format!(
                "MiniMax T2A Async error {}: {}",
                parsed.base_resp.status_code, parsed.base_resp.status_msg
            )));
        }

        Ok(parsed)
    }

    /// Query task status by `task_id`. Returns raw JSON to avoid tight coupling to upstream schema.
    pub async fn get_task_status(
        &self,
        task_id: impl AsRef<str>,
        group_id: &str,
    ) -> Result<Value, AudioGenerationError> {
        // Common pattern: GET /v1/t2a_async_v2/{task_id}?GroupId=...
        let url = format!(
            "{}/{}?GroupId={}",
            T2A_ASYNC_PATH,
            task_id.as_ref(),
            group_id
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(AudioGenerationError::ProviderError(format!(
                "{}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        let text = resp.text().await?;
        if text.is_empty() {
            return Err(AudioGenerationError::ProviderError(
                "Empty status response from MiniMax T2A Async".to_string(),
            ));
        }
        let value: Value = serde_json::from_str(&text)?;
        Ok(value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTaskResponse {
    pub task_id: Option<Value>, // number or string per upstream
    pub task_token: Option<String>,
    pub file_id: Option<Value>,
    pub base_resp: BaseResp,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseResp {
    pub status_code: i64,
    pub status_msg: String,
}
