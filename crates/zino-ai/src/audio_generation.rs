///create a audio_generation trait and component
use crate::client::audio_generation::AudioGenerationModelHandle;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

//Decalaration of the AudioGeneration Error enum
#[derive(Debug, Error)]
pub enum AudioGenerationError {
    /// Http error (e.g.: connection error, timeout, etc.)
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Json error (e.g.: serialization, deserialization)
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error building the transcription request
    #[error("RequestError: {0}")]
    RequestError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Error parsing the transcription response
    #[error("ResponseError: {0}")]
    ResponseError(String),

    /// Error returned by the transcription model provider
    #[error("ProviderError: {0}")]
    ProviderError(String),

    /// Invalid input provided by caller
    #[error("InvalidInput: {0}")]
    InvalidInput(String),
}
// Receive audio generation Response
pub struct AudioGenerationResponse<T> {
    pub audio: Vec<u8>,
    pub metadata: T,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioGenerationRequest {
    pub prompt: String,
    pub input: serde_json::Value,
}

pub struct AudioGenerationRequestBuilder<M: AudioGenerationModel> {
    //Model call
    pub model: M,
    //Text to be converted to audio
    pub text: String,
    //Voice to be used for audio generation
    pub voice: String, //Vec<u8> for optional audio input,
    pub parameters: Value,
}
pub trait AudioGenerationModel: Send + Sync {
    type Response: Send + Sync;
    /// Get the model name.
    fn audio_generation(
        &self,
        request: AudioGenerationRequest,
    ) -> BoxFuture<'static, Result<AudioGenerationResponse<Self::Response>, AudioGenerationError>>;
}
pub trait AudioGeneration<M: AudioGenerationModel>: Send + Sync {
    /// Generate audio from the given model and parameters.
    fn audio_generation(
        &self,
        text: &str,
        voice: &str,
    ) -> impl std::future::Future<
        Output = Result<AudioGenerationRequestBuilder<M>, AudioGenerationError>,
    > + Send;
}
pub trait AudioGenerationModelDyn: Send + Sync {
    /// Get the model name.
    fn audio_generation(
        &self,
        request: AudioGenerationRequest,
    ) -> BoxFuture<'static, Result<AudioGenerationResponse<Value>, AudioGenerationError>>;

    fn audio_generation_request(&self)
    -> AudioGenerationRequestBuilder<AudioGenerationModelHandle>;
}
