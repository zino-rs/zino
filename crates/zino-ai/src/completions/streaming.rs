//! Streaming completion support for AI models.
//!
//! This module provides types and functionality for handling streaming completions,
//! allowing real-time processing of AI model responses as they are generated.

use super::messages::{FunctionResult, ToolCall};
use crate::completions::content::Content;
use crate::completions::messages::Message;
use crate::completions::requests::CompletionError;
use futures::Stream;
use futures::StreamExt;
use futures::stream::{AbortHandle, Abortable};
use std::boxed::Box;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Enum representing either a single value or multiple values.
///
/// This type is useful for APIs that can return either a single item or a collection
/// of items, providing a unified interface for both cases.
#[derive(Debug, Clone)]
pub enum OneOrMany<T> {
    /// A single value.
    One(T),
    /// Multiple values.
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// Create a `OneOrMany::One` from a single item.
    pub fn one(item: T) -> Self {
        OneOrMany::One(item)
    }

    /// Create a `OneOrMany` from a vector of items.
    ///
    /// Returns `None` if the vector is empty, `Some(OneOrMany::One)` if it contains
    /// exactly one item, or `Some(OneOrMany::Many)` if it contains multiple items.
    pub fn many(mut items: Vec<T>) -> Option<Self> {
        match items.len() {
            0 => None,
            1 => Some(OneOrMany::One(items.pop().unwrap())),
            _ => Some(OneOrMany::Many(items)),
        }
    }
}
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
    pub choice: OneOrMany<Message>,
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

impl<R: Clone + Unpin> StreamingCompletionResponse<R> {
    /// Creates a new streaming completion response from a stream.
    ///
    /// # Arguments
    /// * `inner` - The inner stream that produces streaming choices.
    ///
    /// # Returns
    /// A new `StreamingCompletionResponse` instance.
    pub fn stream(inner: StreamingResult<R>) -> StreamingCompletionResponse<R> {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let abortable_stream = Abortable::new(inner, abort_registration);
        Self {
            inner: abortable_stream,
            abort_handle,
            text: "".to_string(),
            tool_calls: vec![],
            choice: OneOrMany::one(Message::Assistant {
                content: Content::text("".to_string()),
                tool_calls: None,
            }),
            response: None,
        }
    }

    /// Cancels the streaming response.
    ///
    /// This method aborts the underlying stream, stopping any further
    /// data from being processed.
    pub fn cancel(&self) {
        self.abort_handle.abort();
    }
}

impl<R: Clone + Unpin> Stream for StreamingCompletionResponse<R> {
    type Item = Result<Message, CompletionError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = self.get_mut();

        match Pin::new(&mut stream.inner).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                // This is run at the end of the inner stream to collect all tokens into
                // a single unified `Message`.
                let mut choice = vec![];

                stream.tool_calls.iter().for_each(|tc| {
                    choice.push(Message::Assistant {
                        content: Content::text("".to_string()),
                        tool_calls: Some(vec![tc.clone()]),
                    });
                });

                // This is required to ensure there's always at least one item in the content
                if choice.is_empty() || !stream.text.is_empty() {
                    choice.insert(
                        0,
                        Message::Assistant {
                            content: Content::text(stream.text.clone()),
                            tool_calls: None,
                        },
                    );
                }

                stream.choice = OneOrMany::many(choice)
                    .expect("There should be at least one assistant message");

                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(err))) => {
                if matches!(err, CompletionError::ProviderError(ref e) if e.to_string().contains("aborted"))
                {
                    return Poll::Ready(None); // Treat cancellation as stream termination
                }
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(Some(Ok(choice))) => match choice {
                RawStreamingChoice::Message(text) => {
                    // Forward the streaming tokens to the outer stream
                    // and concat the text together
                    stream.text = format!("{}{}", stream.text, text.clone());
                    Poll::Ready(Some(Ok(Message::Assistant {
                        content: Content::text(text),
                        tool_calls: None,
                    })))
                }
                RawStreamingChoice::ToolCall {
                    id,
                    name,
                    arguments,
                    call_id: _,
                } => {
                    // Keep track of each tool call to aggregate the final message later
                    // and pass it to the outer stream
                    stream.tool_calls.push(ToolCall {
                        id: id.clone(),
                        r#type: "function".to_string(),
                        function: FunctionResult {
                            name: name.clone(),
                            arguments: arguments.clone(),
                        },
                        mcp: None,
                    });
                    Poll::Ready(Some(Ok(Message::Assistant {
                        content: Content::text("".to_string()),
                        tool_calls: Some(vec![ToolCall {
                            id,
                            r#type: "function".to_string(),
                            function: FunctionResult { name, arguments },
                            mcp: None,
                        }]),
                    })))
                }
                RawStreamingChoice::FinalResponse(response) => {
                    // Set the final response field and return the next item in the stream
                    stream.response = Some(response);

                    stream.poll_next_unpin(cx)
                }
            },
        }
    }
}
