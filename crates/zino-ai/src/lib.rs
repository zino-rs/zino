#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]

pub mod client;
pub mod completions;
pub mod memory_base;
pub mod providers;
pub mod video_generation;

pub mod audio_generation;
pub mod image_generation;
pub mod memory;
pub mod streaming;
pub mod tool;
pub mod transcription;
pub mod workflow;

// Re-export commonly used types
pub use streaming::{OneOrMany, RawStreamingChoice, StreamingCompletionResponse};
