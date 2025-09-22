//! Simplified streaming completion module
//! Provides basic streaming functionality for AI completions

use crate::completions::{CompletionError, Message};
use futures::{Stream, StreamExt};
use serde_json::Value;

/// A simple unit type that implements Clone for use in streaming responses
#[derive(Debug, Clone)]
pub struct EmptyResponse;

/// Simple OneOrMany type for compatibility
#[derive(Debug, Clone)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn one(item: T) -> Self {
        OneOrMany::One(item)
    }
    
    pub fn many(items: Vec<T>) -> Result<Self, &'static str> {
        if items.is_empty() {
            Err("Cannot create OneOrMany::Many from empty vector")
        } else {
            Ok(OneOrMany::Many(items))
        }
    }
}

/// A simple streaming chunk from the model (compatible with existing code)
#[derive(Debug, Clone)]
pub enum RawStreamingChoice<R: Clone> {
    /// Text content
    Message(String),
    /// Tool call
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
        call_id: Option<String>,
    },
    /// Final response
    FinalResponse(R),
}

/// Simple streaming response (compatible with existing code)
pub struct StreamingCompletionResponse<R: Clone> {
    pub stream: Box<dyn Stream<Item = Result<RawStreamingChoice<R>, CompletionError>> + Send + Unpin>,
}

impl<R: Clone> StreamingCompletionResponse<R> {
    pub fn new<S>(stream: S) -> Self 
    where 
        S: Stream<Item = Result<RawStreamingChoice<R>, CompletionError>> + Send + Unpin + 'static
    {
        Self {
            stream: Box::new(stream),
        }
    }

    /// Collect all text from the stream
    pub async fn collect_text(&mut self) -> Result<String, CompletionError> {
        let mut text = String::new();
        
        while let Some(chunk) = self.stream.next().await {
            match chunk? {
                RawStreamingChoice::Message(t) => text.push_str(&t),
                RawStreamingChoice::ToolCall { .. } => {
                    // Skip tool calls for text collection
                }
                RawStreamingChoice::FinalResponse(_) => break,
            }
        }
        
        Ok(text)
    }
}

/// Simple streaming completion trait
pub trait StreamingCompletion {
    /// Generate a streaming completion
    fn stream_completion(
        &self,
        messages: Vec<Message>,
    ) -> impl std::future::Future<Output = Result<StreamingCompletionResponse<()>, CompletionError>> + Send;
}
