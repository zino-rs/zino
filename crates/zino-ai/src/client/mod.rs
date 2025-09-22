pub mod audio_generation;
pub mod completion;
pub mod image_generation;
pub mod transcription;
pub mod video_generation;

pub trait ProviderClient {
    fn from_env() -> Self
    where
        Self: Sized;

    fn boxed(self) -> Box<dyn ProviderClient>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn from_env_boxed<'a>() -> Box<dyn ProviderClient + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(Self::from_env())
    }

    fn from_val(input: String) -> Self
    where
        Self: Sized;

    fn from_val_boxed<'a>(input: String) -> Box<dyn ProviderClient + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(Self::from_val(input))
    }
}

pub trait CompletionClient {
    // fn completion_model(&self, model: &str) -> CompletionModel;
}

pub trait AsAudioGeneration {
    fn as_audio_generation(
        &self,
    ) -> Option<Box<dyn audio_generation::audio::AudioGenerationClientDyn>>;
}

pub trait AsVideoGeneration {
    fn as_video_generation(&self) -> Option<Box<dyn video_generation::VideoGenerationClientDyn>>;
}
pub trait AsImageGeneration {
    fn as_image_generation(
        &self,
    ) -> Option<Box<dyn image_generation::image::ImageGenerationClientDyn>>;
}

pub trait AsTranscription {
    fn as_transcription(&self) -> Option<Box<dyn transcription::TranscriptionClientDyn>>;
}

// pub trait AsCompletion {
//     fn as_completion(&self) -> Option<Box<dyn completion::CompletionClientDyn>>;
// }

#[derive(Clone)]
pub enum ProviderValue {}
