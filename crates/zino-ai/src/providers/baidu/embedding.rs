use super::client::Client;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError {
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
pub struct EmbeddingRequest {
    pub input: InputFormat,
    pub user: Option<String>,
    pub encoding_format: Option<EncodingFormat>,
}

impl EmbeddingRequest {
    /// Create new EmbeddingRequest
    /// Support multiple input types: String, Vec<String>, Vec<MultimodalItem>, Vec<Value> etc.
    pub fn new(message: impl Into<InputFormat>) -> Self {
        Self {
            input: message.into(),
            ..Self::default()
        }
    }

    /// Set user identifier
    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    /// Set encoding format
    pub fn encoding_format(mut self, format: EncodingFormat) -> Self {
        self.encoding_format = Some(format);
        self
    }
}

#[derive(Debug, Clone)]
pub enum InputFormat {
    TEXT(String),
    TEXTARRAY(Vec<String>),
    MULTIMODALARRAY(Vec<MultimodalItem>),
}

impl Serialize for InputFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            InputFormat::TEXT(text) => serializer.serialize_str(text),
            InputFormat::TEXTARRAY(texts) => {
                let mut seq = serializer.serialize_seq(Some(texts.len()))?;
                for text in texts {
                    seq.serialize_element(text)?;
                }
                seq.end()
            }
            InputFormat::MULTIMODALARRAY(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

impl Default for InputFormat {
    fn default() -> Self {
        InputFormat::TEXTARRAY(Vec::new())
    }
}

impl<'de> Deserialize<'de> for InputFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::String(text) => Ok(InputFormat::TEXT(text)),

            Value::Array(arr) => {
                if arr.is_empty() {
                    return Err(serde::de::Error::custom("empty array is not allowed"));
                }

                match &arr[0] {
                    Value::String(_) => {
                        let texts: Vec<String> = arr
                            .into_iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        if texts.is_empty() {
                            return Err(serde::de::Error::custom(
                                "no valid strings found in array",
                            ));
                        }
                        Ok(InputFormat::TEXTARRAY(texts))
                    }
                    Value::Object(_) => {
                        let items: Vec<MultimodalItem> = arr
                            .into_iter()
                            .filter_map(|v| serde_json::from_value(v).ok())
                            .collect();
                        if items.is_empty() {
                            return Err(serde::de::Error::custom(
                                "no valid multimodal items found in array",
                            ));
                        }
                        Ok(InputFormat::MULTIMODALARRAY(items))
                    }
                    _ => Err(serde::de::Error::custom("unsupported array element type")),
                }
            }

            _ => Err(serde::de::Error::custom("unsupported input format")),
        }
    }
}

impl From<String> for InputFormat {
    fn from(text: String) -> Self {
        InputFormat::TEXT(text)
    }
}

impl From<&str> for InputFormat {
    fn from(text: &str) -> Self {
        InputFormat::TEXT(text.to_string())
    }
}

impl From<Vec<String>> for InputFormat {
    fn from(texts: Vec<String>) -> Self {
        InputFormat::TEXTARRAY(texts)
    }
}

impl From<Vec<&str>> for InputFormat {
    fn from(texts: Vec<&str>) -> Self {
        InputFormat::TEXTARRAY(texts.into_iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<Value>> for InputFormat {
    fn from(values: Vec<Value>) -> Self {
        // try to convert Value array to MultimodalItem array
        let items: Vec<MultimodalItem> = values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        if !items.is_empty() {
            InputFormat::MULTIMODALARRAY(items)
        } else {
            // if conversion fails, return an empty text array as default
            InputFormat::TEXTARRAY(vec![])
        }
    }
}

// Implement From for single MultimodalItem
impl From<Value> for InputFormat {
    fn from(value: Value) -> Self {
        if let Ok(item) = serde_json::from_value(value) {
            InputFormat::MULTIMODALARRAY(vec![item])
        } else {
            InputFormat::TEXTARRAY(vec![])
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultimodalItem {
    pub text: Option<String>,
    pub iamge: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum EncodingFormat {
    #[serde(rename = "lowercase")]
    FLOAT,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmbeddingReponse {
    pub model: Option<String>,
    pub id: Option<String>,
    pub pbject: Option<String>,
    pub created: Option<usize>,
    pub data: Option<Vec<Data>>,
    pub usage: Option<Usage>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub object: Option<String>,
    pub embedding: Option<Vec<f64>>,
    pub index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<usize>,
    pub prompt_tokens_details: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
}

pub struct EmbeddingModel {
    pub model: String,
    pub client: Client,
}

impl EmbeddingModel {
    pub fn new(model: &str, client: Client) -> Self {
        Self {
            model: model.to_string(),
            client: client,
        }
    }

    pub fn create_embedding_request(
        &self,
        request: EmbeddingRequest,
    ) -> Result<Value, EmbeddingError> {
        let mut request_body = serde_json::to_value(request)?;
        request_body["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_body)
    }
    pub async fn embedding(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingReponse, EmbeddingError> {
        let request_json = self.create_embedding_request(request)?;
        let response = self
            .client
            .post("v2/embeddings")
            .json(&request_json)
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<EmbeddingReponse>().await?;
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            Err(EmbeddingError::ProviderError(format!(
                "HTTP {}: {}",
                status, error_text
            )))
        }
    }
}
