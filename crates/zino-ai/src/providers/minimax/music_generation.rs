use super::Client;
use crate::audio_generation::AudioGenerationError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioSetting {
    pub sample_rate: u32,
    pub bitrate: u32,
    pub format: String,
}

impl AudioSetting {
    /// Create a new audio setting with default values
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            bitrate: 256000,
            format: "mp3".to_string(),
        }
    }

    /// Create a new audio setting with custom values
    pub fn custom(sample_rate: u32, bitrate: u32, format: String) -> Self {
        Self {
            sample_rate,
            bitrate,
            format,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MusicGenerationRequest {
    pub prompt: String,
    pub lyrics: String,
    pub audio_setting: Option<AudioSetting>,
    pub additional_params: Option<serde_json::Value>,
}
impl MusicGenerationRequest {
    /// Create a new music generation request with required fields
    pub fn new(prompt: String, lyrics: String) -> Self {
        Self {
            prompt,
            lyrics,
            audio_setting: None,
            additional_params: None,
        }
    }

    /// Create a new music generation request with audio settings
    pub fn new_with_settings(prompt: String, lyrics: String, audio_setting: AudioSetting) -> Self {
        Self {
            prompt,
            lyrics,
            audio_setting: Some(audio_setting),
            additional_params: None,
        }
    }

    pub fn add_params(mut self, params: serde_json::Value) -> Self {
        self.additional_params = Some(params);
        self
    }
}

/// Music Generation API Response
#[derive(Debug, Serialize, Deserialize)]
pub struct MusicGenerationResponse {
    /// Response data containing music generation results
    pub data: MusicData,

    // Trace ID for request tracking
    //pub trace_id: String,
    /// Base response containing status information
    pub base_resp: BaseResponse,
}

/// Music generation data
#[derive(Debug, Serialize, Deserialize)]
pub struct MusicData {
    /// Music synthesis status
    /// 1: Synthesizing; 2: Completed
    pub status: u32,

    /// Audio file hex encoded result
    /// Currently only supports generating music within 90 seconds
    pub audio: String,
}

/// Base response containing status information
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseResponse {
    pub status_code: u32,

    /// Status details
    pub status_msg: String,
}

// ================================================================
// Music Generation Model
// ================================================================

/// Music generation model constants
pub const MUSIC_1_5: &str = "music-1.5";

#[derive(Clone)]
pub struct MusicGenerationModel {
    pub(crate) client: Client,
    pub model: String,
}

impl MusicGenerationModel {
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Create a music generation request with required fields
    pub fn create_request(
        &self,
        request: MusicGenerationRequest,
    ) -> Result<serde_json::Value, AudioGenerationError> {
        let mut request_json = serde_json::to_value(&request)?;
        request_json["model"] = serde_json::Value::String(self.model.clone());

        Ok(request_json)
    }
}

impl MusicGenerationModel {
    async fn audio_generation(
        &self,
        request: MusicGenerationRequest,
    ) -> Result<serde_json::Value, AudioGenerationError> {
        // Parse the music generation request from the rig request
        let music_request: Value = self.create_request(request)?;

        // Make the API request
        let response = self
            .client
            .post("/v1/music_generation")
            .json(&music_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AudioGenerationError::ProviderError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let text = response.text().await?;
        if text.is_empty() {
            return Err(AudioGenerationError::ProviderError(
                "Empty response from MiniMax Music Generation API".to_string(),
            ));
        }

        // Parse the response
        let parsed: MusicGenerationResponse = serde_json::from_str(&text).map_err(|e| {
            AudioGenerationError::ProviderError(format!(
                "Failed to parse MiniMax response: {}. Response was: {}",
                e, text
            ))
        })?;

        // Check for API errors
        if parsed.base_resp.status_code != 0 {
            return Err(AudioGenerationError::ProviderError(format!(
                "MiniMax error {}: {}",
                parsed.base_resp.status_code, parsed.base_resp.status_msg
            )));
        }

        // Return the raw API response as JSON
        Ok(serde_json::to_value(parsed)?)
    }
}
