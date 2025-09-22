use super::client::Client;
use crate::image_generation::{ImageGenerationError, ImageGenerationRequest};
use serde::{Deserialize, Serialize};

// MiniMax Image Generation Models
pub const MINIMAX_IMAGE_01: &str = "image-01";
pub const MINIMAX_IMAGE_01_LIVE: &str = "image-01-live";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    pub id: String,
    pub data: ImageData,
    pub metadata: Metadata,
    pub base_resp: BaseResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub image_urls: Option<Vec<String>>,
    pub image_base64: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub success_count: String,
    pub failed_count: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseResponse {
    pub status_code: i32,
    pub status_msg: String,
}

#[derive(Debug, Clone)]
pub struct ImageGenerationModel {
    client: Client,
    model: String,
}

impl ImageGenerationModel {
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

impl crate::image_generation::ImageGenerationModel for ImageGenerationModel {
    type Response = ImageGenerationResponse;

    fn image_generation(
        &self,
        request: ImageGenerationRequest,
    ) -> impl std::future::Future<
        Output = Result<
            crate::image_generation::ImageGenerationResponse<Self::Response>,
            ImageGenerationError,
        >,
    > + Send {
        Box::pin(async move {
            // Convert the generic ImageGenerationRequest to MiniMax specific request
            let mut minimax_request = serde_json::to_value(&request)?;
            minimax_request["model"] = serde_json::Value::String(self.model.clone());

            // Make the API request
            let response = self
                .client
                .post("/v1/image_generation")
                .json(&minimax_request)
                .send()
                .await
                .map_err(ImageGenerationError::HttpError)?;

            if !response.status().is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(ImageGenerationError::ProviderError(error_text).into());
            }

            let minimax_response: ImageGenerationResponse = response
                .json()
                .await
                .map_err(|e| ImageGenerationError::HttpError(e))?;

            // Check if the response indicates an error
            if minimax_response.base_resp.status_code != 0 {
                return Err(ImageGenerationError::ProviderError(
                    minimax_response.base_resp.status_msg,
                )
                .into());
            }

            Ok(crate::image_generation::ImageGenerationResponse {
                image_urls: minimax_response.data.image_urls.clone(),
                image_base64: minimax_response.data.image_base64.clone(),
                response: minimax_response,
            })
        })
    }
}
