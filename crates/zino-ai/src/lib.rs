#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]

pub mod agent;
pub mod client;
pub mod completion;
pub mod embeddings;
pub mod extractor;
pub mod image_generation;
pub mod json_utils;
pub mod memory_base;
pub mod one_or_many;
pub mod streaming;
pub mod tool;
pub mod transcription;
pub mod video_generation;
pub mod audio_generation;
pub mod vector_store;

pub use one_or_many::{EmptyListError, OneOrMany};
// Re-export commonly used types and traits
pub use completion::message;
pub use embeddings::Embed;


