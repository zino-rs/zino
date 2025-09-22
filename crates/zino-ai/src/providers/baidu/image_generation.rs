use super::client::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageGenerationError {
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    //first item should be model,will be mergerd at the post time
    pub prompt: String,
    pub additional_instructions: Option<serde_json::Value>,
}

impl ImageGenerationRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            additional_instructions: None,
        }
    }
    pub fn add_params(mut self, params: serde_json::Value) -> Self {
        self.additional_instructions = Some(params);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ImageGenerationResponse {
    pub id: Option<String>,
    pub created: Option<usize>,
    pub data: Option<Vec<Url>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Url {
    //Image generated url
    pub url: String,
}

pub struct ImageGenerationModel {
    pub model: String,
    pub client: Client,
}

impl ImageGenerationModel {
    pub fn new(model: &str, client: Client) -> Self {
        Self {
            model: model.to_string(),
            client: client,
        }
    }

    pub(crate) fn create_image_generation_request(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<Value, ImageGenerationError> {
        let mut request_body = serde_json::to_value(&request)?;
        request_body["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_body)
    }

    pub async fn image_generation(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, ImageGenerationError> {
        let request_json = self.create_image_generation_request(request)?;
        let response = self
            .client
            .post("/v2/images/generations")
            .json(&request_json)
            .send()
            .await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let text = response.text().await?;

            //
            return Err(ImageGenerationError::ProviderError(format!(
                "HTTP {}: {}",
                status_code, text
            )));
        } else {
            let response_json = response.json::<ImageGenerationResponse>().await?;
            println!("response_Json:{:#?}", &response_json);
            Ok(response_json)
        }
    }

    pub async fn image_generation_edit(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, ImageGenerationError> {
        let request_json = self.create_image_generation_request(request)?;
        let response = self
            .client
            .post("/v2/images/generations")
            .json(&request_json)
            .send()
            .await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let text = response.text().await?;

            //
            return Err(ImageGenerationError::ProviderError(format!(
                "HTTP {}: {}",
                status_code, text
            )));
        } else {
            let response_json = response.json::<ImageGenerationResponse>().await?;
            println!("response_Json:{:#?}", &response_json);
            Ok(response_json)
        }
    }
}
