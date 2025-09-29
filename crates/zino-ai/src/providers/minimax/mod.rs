//! MiniMax AI model provider implementation.
//!
//! This module provides client and completion functionality for MiniMax's
//! AI models, including support for both standard and streaming completions.

/// HTTP client for MiniMax AI API.
pub mod client;

/// Completion models and streaming support for MiniMax AI.
pub mod completion;

/// Re-export commonly used types from client module.
pub use client::{Client, MINIMAX_TEXT_01};

/// Re-export commonly used types from completion module.
pub use completion::{
    CompletionModel, MINIMAX_M1, MINIMAX_TEXT_01 as COMPLETION_MINIMAX_TEXT_01,
    StreamingCompletionResponse, Usage,
};
