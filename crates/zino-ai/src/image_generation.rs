use crate::client::image_generation::ImageGenerationModelHandle;
use futures::future::BoxFuture;
use serde_json::Value;
use std::sync::Arc;
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

pub trait ImageGeneration<M: ImageGenerationModel> {
    /// Generates a transcription request builder for the given `file`.
    /// This function is meant to be called by the user to further customize the
    /// request at transcription time before sending it.
    ///
    /// ❗IMPORTANT: The type that implements this trait might have already
    /// populated fields in the builder (the exact fields depend on the type).
    /// For fields that have already been set by the model, calling the corresponding
    /// method on the builder will overwrite the value set by the model.
    fn image_generation(
        &self,
        prompt: &str,
        size: &(u32, u32),
    ) -> impl std::future::Future<
        Output = Result<ImageGenerationRequestBuilder<M>, ImageGenerationError>,
    > + Send;
}

#[derive(Debug)]
pub struct ImageGenerationResponse<T> {
    pub image_urls: Option<Vec<String>>,
    pub image_base64: Option<Vec<String>>,
    pub response: T,
}

pub trait ImageGenerationModel: Clone + Send + Sync {
    type Response: Send + Sync;

    fn image_generation(
        &self,
        request: ImageGenerationRequest,
    ) -> impl std::future::Future<
        Output = Result<ImageGenerationResponse<Self::Response>, ImageGenerationError>,
    > + Send;

    fn image_generation_request(&self) -> ImageGenerationRequestBuilder<Self> {
        ImageGenerationRequestBuilder::new(self.clone())
    }
}

pub trait ImageGenerationModelDyn: Send + Sync {
    fn image_generation(
        &self,
        request: ImageGenerationRequest,
    ) -> BoxFuture<Result<ImageGenerationResponse<()>, ImageGenerationError>>;

    fn image_generation_request(&self)
    -> ImageGenerationRequestBuilder<ImageGenerationModelHandle>;
}

impl<T: ImageGenerationModel> ImageGenerationModelDyn for T {
    fn image_generation(
        &self,
        request: ImageGenerationRequest,
    ) -> BoxFuture<Result<ImageGenerationResponse<()>, ImageGenerationError>> {
        Box::pin(async {
            let resp = self.image_generation(request).await;
            resp.map(|r| ImageGenerationResponse {
                image_urls: r.image_urls,
                image_base64: r.image_base64,
                response: (),
            })
        })
    }

    fn image_generation_request(
        &self,
    ) -> ImageGenerationRequestBuilder<ImageGenerationModelHandle> {
        ImageGenerationRequestBuilder::new(ImageGenerationModelHandle {
            inner: Arc::new(self.clone()),
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub additional_params: Option<Value>,
}

pub struct ImageGenerationRequestBuilder<M: ImageGenerationModel> {
    model: M,
    prompt: String,
    additional_params: Option<Value>,
}

impl<M: ImageGenerationModel> ImageGenerationRequestBuilder<M> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            prompt: "".to_string(),
            additional_params: None,
        }
    }

    /// Sets the prompt for the image generation request
    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    /// Adds additional parameters to the image generation request.
    pub fn additional_params(mut self, params: Value) -> Self {
        self.additional_params = Some(params);
        self
    }

    pub fn build(self) -> ImageGenerationRequest {
        ImageGenerationRequest {
            prompt: self.prompt,
            additional_params: self.additional_params,
        }
    }

    pub async fn send(self) -> Result<ImageGenerationResponse<M::Response>, ImageGenerationError> {
        let model = self.model.clone();

        model.image_generation(self.build()).await
    }
}
