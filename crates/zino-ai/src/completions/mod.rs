//! Completion types and request handling for AI-powered applications.
//!
//! This module provides core types for handling AI completions,
//! including messages, content, roles, and request/response structures.

/// Content types for AI messages with multimodal support.
pub mod content;

/// Message and content types for conversational AI.
pub mod messages;

/// Request and response types for AI completions.
pub mod requests;

pub mod streaming;

/// Re-export commonly used content types.
pub use content::Content;

/// Re-export request and response types for completion operations.
pub use requests::{CompletionRequest, CompletionResponse};

/// Re-export streaming types for real-time completion handling.
pub use streaming::{RawStreamingChoice, StreamingCompletionResponse};
