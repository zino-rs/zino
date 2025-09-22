use super::client::Client;
use crate::audio_generation::{self, AudioGenerationError, AudioGenerationResponse};
use futures::StreamExt;
use serde::{self, Deserialize, Serialize};
use serde_json::{Value, json};
use std::pin::Pin;

// MiniMax T2A models
pub const SPEECH_2_5_HD_PREVIEW: &str = "speech-2.5-hd-preview";
pub const SPEECH_2_5_TURBO_PREVIEW: &str = "speech-2.5-turbo-preview";
pub const SPEECH_02_HD: &str = "speech-02-hd";
pub const SPEECH_02_TURBO: &str = "speech-02-turbo";
pub const SPEECH_01_HD: &str = "speech-01-hd";
pub const SPEECH_01_TURBO: &str = "speech-01-turbo";

#[derive(Clone)]
pub struct AudioGenerationModel {
    client: Client,
    pub model: String,
}

impl AudioGenerationModel {
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioGenerationRequest {
    pub text: Option<String>,
    pub voice_setting: Option<VoiceType>,
    pub audio_setting: Option<VoiceType>,
    pub pronunciation_dict: Option<Pronun>,
    pub timber_weights: Option<Timber>,
    pub stream: Option<bool>,
    pub stream_options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timber {
    pub voice_id: Option<String>,
    pub weight: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pronun {
    pub tone: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceType {
    pub speed: Option<f32>,
    pub vol: Option<f32>,
    pub pitch: Option<i32>,
    pub voice_id: Option<String>,
    pub emotion: Option<String>,
    pub latex_read: Option<bool>,
    pub english_normalization: Option<bool>,
}

// Response wrapper for MiniMax T2A
#[derive(Debug, serde::Deserialize)]
struct T2AResponse {
    #[serde(default)]
    data: Option<T2AData>,
    #[serde(default)]
    extra_info: Option<Value>,
    #[serde(default)]
    trace_id: Option<String>,
    #[serde(default)]
    base_resp: Option<BaseResp>,
}

#[derive(Debug, serde::Deserialize)]
struct T2AData {
    #[serde(default)]
    audio: Option<String>, // hex or url
    #[serde(default)]
    status: Option<i32>, // 1 processing, 2 done
}

#[derive(Debug, serde::Deserialize)]
struct BaseResp {
    status_code: i64,
    #[serde(default)]
    status_msg: String,
}

fn merge_model_into_payload(model: &str, input: &Value) -> Value {
    // MiniMax expects flat schema, not nested { input: ... }
    // We merge user input fields with model; the user should provide remaining fields like text, voice_setting, etc.
    let mut body = json!({ "model": model });
    if let Some(obj) = input.as_object() {
        let mut merged = body.as_object_mut().unwrap().clone();
        for (k, v) in obj.iter() {
            merged.insert(k.clone(), v.clone());
        }
        body = Value::Object(merged);
    }
    body
}

fn decode_hex_audio(hex_str: &str) -> Result<Vec<u8>, AudioGenerationError> {
    // Remove whitespace/newlines
    let clean: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
    // If it's clearly a URL, fallback to URL handling by caller
    if clean.starts_with("http://") || clean.starts_with("https://") {
        return Err(AudioGenerationError::ResponseError(
            "Expected hex audio but got URL".to_string(),
        ));
    }
    // Hex decode
    let bytes = (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AudioGenerationError::ResponseError(format!("Invalid hex audio: {}", e)))?;
    Ok(bytes)
}

async fn fetch_url_bytes(_client: &Client, url: &str) -> Result<Vec<u8>, AudioGenerationError> {
    let bytes = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .bytes()
        .await?;
    Ok(bytes.to_vec())
}

fn build_t2a_url(group_id: Option<&str>) -> String {
    let base = "/v1/t2a_v2";
    if let Some(gid) = group_id {
        format!("{}?GroupId={}", base, gid)
    } else {
        base.to_string()
    }
}

impl audio_generation::AudioGenerationModel for AudioGenerationModel {
    type Response = serde_json::Value;

    fn audio_generation(
        &self,
        request: audio_generation::AudioGenerationRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<AudioGenerationResponse<Self::Response>, AudioGenerationError>,
                > + Send,
        >,
    > {
        let this = self.clone();
        let req = request.clone();
        Box::pin(async move {
            // Convert AudioGenerationRequest to JSON
            let mut body = serde_json::to_value(&req.input).unwrap_or_else(|_| json!({}));
            body["model"] = serde_json::Value::String(this.model.clone());

            // Extract optional GroupId from input; accept keys: group_id or GroupId
            let group_id = body
                .get("group_id")
                .and_then(|v| v.as_str())
                .or_else(|| body.get("GroupId").and_then(|v| v.as_str()))
                .map(|s| s.to_string());
            if body.as_object_mut().unwrap().contains_key("group_id") {
                body.as_object_mut().unwrap().remove("group_id");
            }
            if body.as_object_mut().unwrap().contains_key("GroupId") {
                body.as_object_mut().unwrap().remove("GroupId");
            }

            // Ensure output_format defaults to hex for easier decoding (non-stream)
            if !body.get("output_format").is_some() {
                // Merge output_format into body
                if let Some(body_obj) = body.as_object_mut() {
                    body_obj.insert("output_format".to_string(), json!("hex"));
                }
            }

            // Use stream flag to decide streaming or non-stream
            let is_stream = body
                .get("stream")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let path = build_t2a_url(group_id.as_deref());
            let builder = this.client.post(&path).json(&body);

            if !is_stream {
                let response = builder.send().await?;
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
                        "Empty response from MiniMax T2A".to_string(),
                    ));
                }
                let parsed: T2AResponse = serde_json::from_str(&text).map_err(|e| {
                    AudioGenerationError::ProviderError(format!(
                        "Failed to parse T2A response: {}. Response was: {}",
                        e, text
                    ))
                })?;

                if let Some(base) = &parsed.base_resp {
                    if base.status_code != 0 {
                        return Err(AudioGenerationError::ProviderError(format!(
                            "MiniMax T2A error {}: {}",
                            base.status_code, base.status_msg
                        )));
                    }
                }

                let mut audio_bytes: Vec<u8> = vec![];
                if let Some(data) = parsed.data {
                    if let Some(audio_str) = data.audio {
                        if audio_str.starts_with("http://") || audio_str.starts_with("https://") {
                            audio_bytes = fetch_url_bytes(&this.client, &audio_str).await?;
                        } else {
                            audio_bytes = decode_hex_audio(&audio_str)?;
                        }
                    }
                }

                Ok(AudioGenerationResponse {
                    audio: audio_bytes,
                    metadata: serde_json::to_value(parsed.extra_info).unwrap_or(Value::Null),
                })
            } else {
                // Streaming: aggregate audio hex chunks until status 2
                let response = builder.send().await?;
                if !response.status().is_success() {
                    return Err(AudioGenerationError::ProviderError(format!(
                        "{}: {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    )));
                }

                // Since bytes_stream requires 'stream' feature, use a different approach
                let response_text = response.text().await?;
                let mut stream =
                    futures::stream::once(async { Ok(bytes::Bytes::from(response_text)) }).boxed();
                let mut buffer = String::new();
                let mut audio_hex_concat = String::new();

                while let Some(chunk_result) = stream.next().await {
                    let bytes = chunk_result.map_err(AudioGenerationError::HttpError)?;
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);

                    // Process by lines
                    loop {
                        if let Some(pos) = buffer.find('\n') {
                            let mut line = buffer[..pos].trim().to_string();
                            buffer.drain(..=pos);
                            if line.is_empty() {
                                continue;
                            }

                            // Remove SSE prefix if present
                            if let Some(rest) = line.strip_prefix("data:") {
                                line = rest.trim().to_string();
                            }

                            // Try parse JSON line
                            match serde_json::from_str::<T2AResponse>(&line) {
                                Ok(part) => {
                                    if let Some(base) = &part.base_resp {
                                        if base.status_code != 0 {
                                            return Err(AudioGenerationError::ProviderError(
                                                format!(
                                                    "MiniMax T2A stream error {}: {}",
                                                    base.status_code, base.status_msg
                                                ),
                                            ));
                                        }
                                    }
                                    if let Some(data) = part.data {
                                        if let Some(audio_str) = data.audio {
                                            audio_hex_concat.push_str(&audio_str);
                                        }
                                        if matches!(data.status, Some(2)) {
                                            // End
                                            let audio = decode_hex_audio(&audio_hex_concat)?;
                                            return Ok(AudioGenerationResponse {
                                                audio,
                                                metadata: serde_json::Value::Null,
                                            });
                                        }
                                    }
                                }
                                Err(_) => {
                                    // skip unparsable line
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }

                // If we exit the loop without status 2, try decode what we have
                let audio = if !audio_hex_concat.is_empty() {
                    decode_hex_audio(&audio_hex_concat)?
                } else {
                    vec![]
                };
                Ok(AudioGenerationResponse {
                    audio,
                    metadata: serde_json::Value::Null,
                })
            }
        })
    }
}

impl audio_generation::AudioGenerationModelDyn for AudioGenerationModel {
    fn audio_generation(
        &self,
        request: audio_generation::AudioGenerationRequest,
    ) -> futures::future::BoxFuture<
        'static,
        Result<AudioGenerationResponse<serde_json::Value>, AudioGenerationError>,
    > {
        let this = self.clone();
        let req = request.clone();
        Box::pin(async move {
            // Convert the request to the correct type
            let rig_request = audio_generation::AudioGenerationRequest {
                prompt: req.prompt,
                input: req.input,
            };
            this.audio_generation(rig_request).await
        })
    }

    fn audio_generation_request(
        &self,
    ) -> crate::audio_generation::AudioGenerationRequestBuilder<
        crate::client::audio_generation::AudioGenerationModelHandle,
    > {
        crate::audio_generation::AudioGenerationRequestBuilder {
            model: crate::client::audio_generation::AudioGenerationModelHandle {
                inner: std::sync::Arc::new(self.clone()),
            },
            text: String::new(),
            voice: String::new(),
            parameters: serde_json::json!({}),
        }
    }
}
