mod audio {
    use crate::audio_generation::{
        AudioGenerationError, AudioGenerationModel, AudioGenerationModelDyn,
        AudioGenerationRequest, AudioGenerationResponse,
    };
    use crate::client::{AsAudioGeneration, ProviderClient};

    use std::sync::Arc;

    /// A provider client with audio generation capabilities.
    /// Clone is required for conversions between client types.
    pub trait AudioGenerationClient: ProviderClient + Clone {
        /// The AudioGenerationModel used by the Client
        type AudioGenerationModel: AudioGenerationModel;

        /// Create an audio generation model with the given name.
        ///
        /// # Example
        /// ```
        /// use rig::providers::openai::{Client, self};
        ///
        /// // Initialize the OpenAI client
        /// let openai = Client::new("your-open-ai-api-key");
        ///
        /// let tts = openai.audio_generation_model(openai::TTS_1);
        /// ```
        fn audio_generation_model(&self, model: &str) -> Self::AudioGenerationModel;
    }

    pub trait AudioGenerationClientDyn: ProviderClient {
        fn audio_generation_model<'a>(&self, model: &str) -> Box<dyn AudioGenerationModelDyn + 'a>;
    }

    impl<
        T: AudioGenerationClient<AudioGenerationModel = M>,
        M: AudioGenerationModel + crate::audio_generation::AudioGenerationModelDyn + 'static,
    > AudioGenerationClientDyn for T
    {
        fn audio_generation_model<'a>(&self, model: &str) -> Box<dyn AudioGenerationModelDyn + 'a> {
            Box::new(self.audio_generation_model(model))
        }
    }

    impl<T: AudioGenerationClientDyn + Clone + 'static> AsAudioGeneration for T {
        fn as_audio_generation(&self) -> Option<Box<dyn AudioGenerationClientDyn>> {
            Some(Box::new(self.clone()))
        }
    }

    /// Wraps a AudioGenerationModel in a dyn-compatible way for AudioGenerationRequestBuilder.
    #[derive(Clone)]
    pub struct AudioGenerationModelHandle<'a> {
        pub(crate) inner: Arc<dyn AudioGenerationModelDyn + 'a>,
    }
    impl AudioGenerationModel for AudioGenerationModelHandle<'_> {
        type Response = serde_json::Value;

        fn audio_generation(
            &self,
            request: AudioGenerationRequest,
        ) -> futures::future::BoxFuture<
            'static,
            Result<AudioGenerationResponse<Self::Response>, AudioGenerationError>,
        > {
            self.inner.audio_generation(request)
        }
    }
}

pub use audio::*;
