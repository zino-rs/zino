//use crate::client::video_generation::VideoGenerationModelHandle;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
//use std::sync::Arc;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum VideoGenerationError {
    /// Http error (e.g.: connection error, timeout, etc.)
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Json error (e.g.: serialization, deserialization)
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error building the video generation request
    #[error("RequestError: {0}")]
    RequestError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Error parsing the video generation response
    #[error("ResponseError: {0}")]
    ResponseError(String),

    /// Error returned by the video generation model provider
    #[error("ProviderError: {0}")]
    ProviderError(String),

    /// Invalid input parameters
    #[error("InvalidInput: {0}")]
    InvalidInput(String),

    /// Task timeout
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Generic trait for video generation models
pub trait VideoGenerationModel: Clone + Send + Sync {
    /// Execute video generation request
    fn video_generation(
        &self,
        request: VideoGenerationRequest,
    ) -> impl std::future::Future<Output = Result<VideoGenerationResponse, VideoGenerationError>> + Send;

    /// creating a video generation request builder
    fn video_generation_request(&self) -> VideoGenerationRequestBuilder<Self> {
        VideoGenerationRequestBuilder::new(self.clone())
    }
}

/// Dynamic dispatch trait
pub trait VideoGenerationModelDyn: Send + Sync {
    fn video_generation(
        &self,
        request: VideoGenerationRequest,
    ) -> BoxFuture<'_, Result<VideoGenerationResponse, VideoGenerationError>>;

    // fn video_generation_request(
    //     &self,
    // ) -> VideoGenerationRequestBuilder<VideoGenerationModelHandle<'_>>;
}

impl<T: VideoGenerationModel> VideoGenerationModelDyn for T {
    fn video_generation(
        &self,
        request: VideoGenerationRequest,
    ) -> BoxFuture<'_, Result<VideoGenerationResponse, VideoGenerationError>> {
        Box::pin(async move {
            let resp = self.video_generation(request).await?;
            Ok(VideoGenerationResponse {
                task_id: resp.task_id,
                video_url: resp.video_url,
            })
        })
    }

    // fn video_generation_request(
    //     &self,
    // ) -> VideoGenerationRequestBuilder<VideoGenerationModelHandle<'_>> {
    //     VideoGenerationRequestBuilder::new(VideoGenerationModelHandle {
    //         inner: Arc::new(self.clone()),
    //     })
    // }
}

/// create video generation request with custom parameters
#[derive(Debug, Serialize, Clone)]
pub struct VideoGenerationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_params: Option<Value>,
}

/// generic video generation response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoGenerationResponse {
    pub task_id: Option<String>,

    pub video_url: Option<Vec<String>>,
}

/// generic video generation request builder
pub struct VideoGenerationRequestBuilder<M: VideoGenerationModel> {
    model: M,
    custom_params: Option<Value>,
}

impl<M: VideoGenerationModel> VideoGenerationRequestBuilder<M> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            custom_params: None,
        }
    }

    /// set custom parameters as serde_json::Value
    pub fn custom_params(mut self, params: Value) -> Self {
        self.custom_params = Some(params);
        self
    }

    /// build video generation request
    pub fn build(self) -> VideoGenerationRequest {
        VideoGenerationRequest {
            custom_params: self.custom_params,
        }
    }

    ///
    pub async fn send(self) -> Result<VideoGenerationResponse, VideoGenerationError> {
        let model = self.model.clone();
        model.video_generation(self.build()).await
    }
}
