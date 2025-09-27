//! Streaming completion support for AI models.
//!
//! This module provides types and functionality for handling streaming completions,
//! allowing real-time processing of AI model responses as they are generated.

use super::messages::ToolCall;
use crate::completions::messages::Message;
use crate::completions::requests::CompletionError;
use futures::Stream;
use futures::stream::{AbortHandle, Abortable};
use std::boxed::Box;
use std::pin::Pin;

/// Enum representing different types of streaming chunks from the model.
///
/// This enum captures the various types of data that can be streamed from an AI model,
/// including text chunks, tool calls, and final responses.
#[derive(Debug, Clone)]
pub enum RawStreamingChoice<R: Clone> {
    /// A text chunk from a message response.
    /// This represents a piece of text content that is part of the model's response.
    Message(String),

    /// A tool call response chunk.
    /// This represents a tool call that the model wants to make, including
    /// the function name, arguments, and optional metadata.
    ToolCall {
        /// Unique identifier for this tool call.
        id: String,
        /// Optional call ID for tracking the tool call.
        call_id: Option<String>,
        /// Name of the function to be called.
        name: String,
        /// Arguments for the function call as JSON.
        arguments: serde_json::Value,
    },

    /// The final response object.
    /// This must be yielded if you want the `response` field to be populated
    /// on the `StreamingCompletionResponse`.
    FinalResponse(R),
}

/// The response from a streaming completion request.
///
/// This struct contains the streaming data and aggregated results from a streaming
/// completion request. The message and response fields are populated at the end of
/// the inner stream.
pub struct StreamingCompletionResponse<R: Clone + Unpin> {
    /// The abortable stream of streaming choices.
    #[allow(dead_code)]
    pub(crate) inner: Abortable<StreamingResult<R>>,
    /// Handle for aborting the stream if needed.
    #[allow(dead_code)]
    pub(crate) abort_handle: AbortHandle,
    /// Accumulated text content from the stream.
    #[allow(dead_code)]
    text: String,
    /// Accumulated tool calls from the stream.
    #[allow(dead_code)]
    tool_calls: Vec<ToolCall>,
    /// The final aggregated message from the stream.
    /// Contains all text and tool calls generated during the stream.
    pub choice: Vec<Message>,
    /// The final response from the stream.
    /// May be `None` if the provider didn't yield it during the stream.
    pub response: Option<R>,
}

/// Type alias for a streaming result that yields completion choices or errors.
///
/// This represents a stream that produces `RawStreamingChoice` items or `CompletionError`s
/// as the AI model generates responses in real-time.
pub type StreamingResult<R> =
    Pin<Box<dyn Stream<Item = Result<RawStreamingChoice<R>, CompletionError>> + Send>>;
