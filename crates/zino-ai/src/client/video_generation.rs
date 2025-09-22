use crate::client::{AsVideoGeneration, ProviderClient};
use crate::video_generation::{
    VideoGenerationError, VideoGenerationModel, VideoGenerationModelDyn, VideoGenerationRequest,
    VideoGenerationResponse,
};
use std::sync::Arc;

pub trait VideoGenerationClient: ProviderClient + Clone {
    /// The type of VideoGenerationModel used by the Client
    type VideoGenerationModel: VideoGenerationModel;

    fn video_generation_model(&self, model: &str) -> Self::VideoGenerationModel;
}

pub trait VideoGenerationClientDyn: ProviderClient {
    /// Create a video generation model with the given name.
    fn video_generation_model<'a>(&self, model: &str) -> Box<dyn VideoGenerationModelDyn + 'a>;
}

impl<T: VideoGenerationClient<VideoGenerationModel = M>, M: VideoGenerationModel + 'static>
    VideoGenerationClientDyn for T
{
    fn video_generation_model<'a>(&self, model: &str) -> Box<dyn VideoGenerationModelDyn + 'a> {
        Box::new(self.video_generation_model(model))
    }
}

impl<T: VideoGenerationClientDyn + Clone + 'static> AsVideoGeneration for T {
    fn as_video_generation(&self) -> Option<Box<dyn VideoGenerationClientDyn>> {
        Some(Box::new(self.clone()))
    }
}
/// Wraps a VideoGenerationModel in a dyn-compatible way for VideoGenerationRequestBuilder.
#[derive(Clone)]
pub struct VideoGenerationModelHandle<'a> {
    pub inner: Arc<dyn VideoGenerationModelDyn + 'a>,
}

impl VideoGenerationModel for VideoGenerationModelHandle<'_> {
    async fn video_generation(
        &self,
        request: VideoGenerationRequest,
    ) -> Result<VideoGenerationResponse, VideoGenerationError> {
        self.inner.video_generation(request).await
    }
}
