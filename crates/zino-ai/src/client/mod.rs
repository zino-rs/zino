//! This module provides traits for defining and creating provider clients.
//! Clients are used to create models for completion, embeddings, etc.
//! Dyn-compatible traits have been provided to allow for more provider-agnostic code.

pub mod audio_generation;
pub mod builder;
pub mod completion;
pub mod embeddings;
pub mod image_generation;
pub mod transcription;
pub mod video_generation;

#[cfg(feature = "derive")]
pub use rig_derive::ProviderClient;
use std::fmt::Debug;

/// The base ProviderClient trait, facilitates conversion between client types
/// and creating a client from the environment.
///
/// All conversion traits must be implemented, they are automatically
/// implemented if the respective client trait is implemented.
pub trait ProviderClient:
    AsCompletion + AsTranscription + AsEmbeddings + AsImageGeneration + AsAudioGeneration + Debug
{
    /// Create a client from the process's environment.
    /// Panics if an environment is improperly configured.
    fn from_env() -> Self
    where
        Self: Sized;

    /// A helper method to box the client.
    fn boxed(self) -> Box<dyn ProviderClient>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    /// Create a boxed client from the process's environment.
    /// Panics if an environment is improperly configured.
    fn from_env_boxed<'a>() -> Box<dyn ProviderClient + 'a>
    where
        Self: Sized,
        Self: 'a,
    {
        Box::new(Self::from_env())
    }

    fn from_val(input: ProviderValue) -> Self
    where
        Self: Sized;

    /// Create a boxed client from the process's environment.
    /// Panics if an environment is improperly configured.
    fn from_val_boxed<'a>(input: ProviderValue) -> Box<dyn ProviderClient + 'a>
    where
        Self: Sized,
        Self: 'a,
    {
        Box::new(Self::from_val(input))
    }
}

#[derive(Clone)]
pub enum ProviderValue {
    Simple(String),
    ApiKeyWithOptionalKey(String, Option<String>),
    ApiKeyWithVersionAndHeader(String, String, String),
}

impl From<&str> for ProviderValue {
    fn from(value: &str) -> Self {
        Self::Simple(value.to_string())
    }
}

impl From<String> for ProviderValue {
    fn from(value: String) -> Self {
        Self::Simple(value)
    }
}

impl<P> From<(P, Option<P>)> for ProviderValue
where
    P: AsRef<str>,
{
    fn from((api_key, optional_key): (P, Option<P>)) -> Self {
        Self::ApiKeyWithOptionalKey(
            api_key.as_ref().to_string(),
            optional_key.map(|x| x.as_ref().to_string()),
        )
    }
}

impl<P> From<(P, P, P)> for ProviderValue
where
    P: AsRef<str>,
{
    fn from((api_key, version, header): (P, P, P)) -> Self {
        Self::ApiKeyWithVersionAndHeader(
            api_key.as_ref().to_string(),
            version.as_ref().to_string(),
            header.as_ref().to_string(),
        )
    }
}

/// Attempt to convert a ProviderClient to a CompletionClient
pub trait AsCompletion {
    fn as_completion(&self) -> Option<Box<dyn CompletionClientDyn>> {
        None
    }
}

/// Attempt to convert a ProviderClient to a TranscriptionClient
pub trait AsTranscription {
    fn as_transcription(&self) -> Option<Box<dyn TranscriptionClientDyn>> {
        None
    }
}

/// Attempt to convert a ProviderClient to a EmbeddingsClient
pub trait AsEmbeddings {
    fn as_embeddings(&self) -> Option<Box<dyn EmbeddingsClientDyn>> {
        None
    }
}

/// Attempt to convert a ProviderClient to a AudioGenerationClient
pub trait AsAudioGeneration {
    fn as_audio_generation(
        &self,
    ) -> Option<Box<dyn crate::client::audio_generation::AudioGenerationClientDyn>> {
        None
    }
}

/// Attempt to convert a ProviderClient to a ImageGenerationClient
pub trait AsImageGeneration {
    fn as_image_generation(&self) -> Option<Box<dyn ImageGenerationClientDyn>> {
        None
    }
}

/// Attempt to convert a ProviderClient to a VideoGenerationClient
pub trait AsVideoGeneration {
    fn as_video_generation(&self) -> Option<Box<dyn VideoGenerationClientDyn>> {
        None
    }
}
// #[cfg(not(feature = "audio"))]
// impl<T: ProviderClient> AsAudioGeneration for T {}

// #[cfg(not(feature = "image"))]
// impl<T: ProviderClient> AsImageGeneration for T {}

/// Implements the conversion traits for a given struct
/// ```rust
/// pub struct Client;
/// impl ProviderClient for Client {
///     ...
/// }
/// impl_conversion_traits!(AsCompletion, AsEmbeddings for Client);
/// ```
#[macro_export]
macro_rules! impl_conversion_traits {
    ($( $trait_:ident ),* for $struct_:ident ) => {
        $(
            impl_conversion_traits!(@impl $trait_ for $struct_);
        )*
    };

    (@impl AsAudioGeneration for $struct_:ident ) => {
        rig::client::impl_audio_generation!($struct_);
    };

    (@impl AsImageGeneration for $struct_:ident ) => {
        rig::client::impl_image_generation!($struct_);
    };

    (@impl $trait_:ident for $struct_:ident) => {
        impl rig::client::$trait_ for $struct_ {}
    };

    (@impl AsVideoGeneration for $struct_:ident) => {
        rig::client::impl_video_generation!($struct_);
    };
}

#[macro_export]
macro_rules! impl_audio_generation {
    ($struct_:ident) => {
        impl rig::client::AsAudioGeneration for $struct_ {}
    };
}

#[macro_export]
macro_rules! impl_image_generation {
    ($struct_:ident) => {
        impl rig::client::AsImageGeneration for $struct_ {}
    };
}

#[macro_export]
macro_rules! impl_video_generation {
    ($struct_:ident) => {
        impl rig::client::AsVideoGeneration for $struct_ {}
    };
}

pub use impl_audio_generation;
pub use impl_conversion_traits;
pub use impl_image_generation;
pub use impl_video_generation;

use crate::client::completion::CompletionClientDyn;
use crate::client::embeddings::EmbeddingsClientDyn;
use crate::client::image_generation::ImageGenerationClientDyn;
use crate::client::transcription::TranscriptionClientDyn;
use crate::client::video_generation::VideoGenerationClientDyn;

pub use crate::client::audio_generation::AudioGenerationClient;
pub use crate::client::completion::CompletionClient;
pub use crate::client::embeddings::EmbeddingsClient;
pub use crate::client::image_generation::ImageGenerationClient;
pub use crate::client::transcription::TranscriptionClient;
pub use crate::client::video_generation::VideoGenerationClient;
