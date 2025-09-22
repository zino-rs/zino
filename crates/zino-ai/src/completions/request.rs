use super::messages::Message;
use crate::streaming::StreamingCompletionResponse;
use std::ops::{Add, AddAssign};
use thiserror::Error;
// use futures::StreamExt;  // Commented out as it's unused

// Errors
#[derive(Debug, Error)]
pub enum CompletionError {
    /// Http error (e.g.: connection error, timeout, etc.)
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Json error (e.g.: serialization, deserialization)
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error building the completion request
    #[error("RequestError: {0}")]
    RequestError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Error parsing the completion response
    #[error("ResponseError: {0}")]
    ResponseError(String),

    /// Error returned by the completion model provider
    #[error("ProviderError: {0}")]
    ProviderError(String),
}

#[derive(Debug)]
pub struct CompletionResponse<T> {
    /// The completion choice (represented by one or more assistant message content)
    /// returned by the completion model provider
    pub choice: Vec<Message>,
    /// Tokens used during prompting and responding
    pub usage: Usage,
    /// The raw response returned by the completion model provider
    pub raw_response: T,
}

/// Trait defining a completion model that can be used to generate completion responses.
/// This trait is meant to be implemented by the user to define a custom completion model,
/// either from a third party provider (e.g.: OpenAI) or a local model.
pub trait CompletionModel: Clone + Send + Sync {
    /// The raw response type returned by the underlying completion model.
    type Response: Send + Sync;
    /// The raw response type returned by the underlying completion model when streaming.
    type StreamingResponse: Clone + Unpin + Send + Sync;

    /// Generates a completion response for the given completion request.
    fn completion(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<Output = Result<serde_json::Value, CompletionError>> + Send;

    /// Generates a completion request builder for the given `prompt`.
    fn completion_request(&self, prompt: impl Into<Message>) -> CompletionRequestBuilder<Self> {
        CompletionRequestBuilder::new(self.clone(), prompt)
    }

    /// Streams a completion response for the given completion request.
    fn stream(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<
        Output = Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>,
    > + Send;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionRequest {
    // message is required
    pub messages: Vec<Message>,
    // user can optionally provide other parameters according to different providers
    pub additional_kwargs: Option<serde_json::Value>,
}

/// Struct representing the token usage for a completion request.
/// If tokens used are `0`, then the provider failed to supply token usage metrics.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    // We store this separately as some providers may only report one number
    pub total_tokens: u64,
}

impl Usage {
    pub fn new() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
        }
    }
}

impl Default for Usage {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for Usage {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            input_tokens: self.input_tokens + other.input_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
        }
    }
}

impl AddAssign for Usage {
    fn add_assign(&mut self, other: Self) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
    }
}

pub struct CompletionRequestBuilder<M: CompletionModel> {
    model: M,
    prompt: Message,
    preamble: Option<String>,
    chat_history: Vec<Message>,
    additional_params: Option<serde_json::Value>,
}

impl<M: CompletionModel> CompletionRequestBuilder<M> {
    pub fn new(model: M, prompt: impl Into<Message>) -> Self {
        Self {
            model,
            prompt: prompt.into(),
            preamble: None,
            chat_history: Vec::new(),
            additional_params: None,
        }
    }

    /// Sets the preamble for the completion request.
    pub fn preamble(mut self, preamble: String) -> Self {
        self.preamble = Some(preamble);
        self
    }

    /// Adds a message to the chat history for the completion request.
    pub fn message(mut self, message: Message) -> Self {
        self.chat_history.push(message);
        self
    }

    /// Adds a list of messages to the chat history for the completion request.
    pub fn messages(self, messages: Vec<Message>) -> Self {
        messages
            .into_iter()
            .fold(self, |builder, msg| builder.message(msg))
    }

    pub fn additional_params(mut self, additional_params: serde_json::Value) -> Self {
        // match self.additional_params {
        //     Some(params) => {
        //         self.additional_params = Some(json_utils::merge(params, additional_params));
        //     }
        //     None => {
        //         self.additional_params = Some(additional_params);
        //     }

        // }
        self.additional_params = Some(additional_params);
        self
    }

    /// Sets the additional parameters for the completion request.
    /// This can be used to set additional provider-specific parameters. For example,
    /// Cohere's completion models accept a `connectors` parameter that can be used to
    /// specify the data connectors used by Cohere when executing the completion
    /// (see `examples/cohere_connectors.rs`).
    pub fn additional_params_opt(mut self, additional_params: Option<serde_json::Value>) -> Self {
        self.additional_params = additional_params;
        self
    }

    // Builds the completion request.
    // pub fn build(self) -> CompletionRequest {
    //     let chat_history = OneOrMany::many([self.chat_history, vec![self.prompt]].concat())
    //         .expect("There will always be atleast the prompt");

    //     CompletionRequest {
    //         preamble: self.preamble,
    //         chat_history,
    //         additional_params: self.additional_params,
    //     }
    // }

    // Sends the completion request to the completion model provider and returns the completion response.
    // pub async fn send(self) -> Result<CompletionResponse<M::Response>, CompletionError> {
    //     let model = self.model.clone();
    //     model.completion(self.build()).await
    // }

    // Stream the completion request
    // pub async fn stream<'a>(
    //     self,
    // ) -> Result<StreamingCompletionResponse<M::StreamingResponse>, CompletionError>
    // where
    //     <M as CompletionModel>::StreamingResponse: 'a,
    //     Self: 'a,
    // {
    //     let model = self.model.clone();
    //     model.stream(self.build()).await
    // }
}

//Define a general CompletionResponse for all providers
pub struct CompletionResult {
    pub choice: Vec<Message>,
    pub usage: Usage,
    pub raw_response: serde_json::Value,
}
