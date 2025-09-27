use super::StreamingCompletionResponse;
use super::messages::Message;
use reqwest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during completion operations.
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

/// Completion request structure for AI model interactions.
///
/// `CompletionRequest` represents a request to an AI model for text completion.
/// It includes the conversation messages, optional tools, and additional parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The conversation messages to send to the AI model.
    pub messages: Vec<Message>,

    /// Optional tools that the AI model can use during the conversation.
    pub tools: Option<Vec<Tool>>,

    /// Additional parameters for the completion request.
    /// These can include model-specific settings like temperature, max_tokens, etc.
    pub additional_params: Option<serde_json::Value>,
}

/// Tool definition for AI model interactions.
///
/// `Tool` represents a tool that can be used by the AI model during conversations.
/// Tools allow the model to call external functions or APIs to perform specific tasks.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    /// The type of tool, typically "function".
    #[serde(rename = "type", default = "default_tool_type")]
    pub r#type: String,

    /// The function definition for this tool.
    #[serde(rename = "function")]
    pub function: Function,
}

fn default_tool_type() -> String {
    "function".to_string()
}

impl Tool {
    /// Creates a new tool with the specified type and function.
    ///
    /// # Arguments
    /// * `tool_type` - The type of tool (e.g., "function")
    /// * `function` - The function definition for this tool
    ///
    /// # Returns
    /// * `Tool` - The created tool
    pub fn new(tool_type: String, function: Function) -> Self {
        Self {
            r#type: tool_type,
            function,
        }
    }

    /// Creates a new tool with default type "function".
    ///
    /// # Arguments
    /// * `function` - The function definition for this tool
    ///
    /// # Returns
    /// * `Tool` - The created tool with type "function"
    pub fn new_function(function: Function) -> Self {
        Self {
            r#type: "function".to_string(),
            function,
        }
    }
}
/// Function definition for tools.
///
/// `Function` represents a function that can be called by the AI model.
/// It includes the function name, description, and parameter schema.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    /// The name of the function.
    #[serde(rename = "name")]
    pub name: String,

    /// A description of what the function does.
    #[serde(rename = "description")]
    pub description: String,

    /// JSON schema defining the function parameters.
    #[serde(rename = "parameters")]
    pub parameters: serde_json::Value,
}

/// General completion response struct that contains the high-level completion choice
/// and the raw response. The completion choice contains one or more assistant content.
#[derive(Debug)]
pub struct CompletionResponse<T> {
    /// The raw response returned by the completion model provider
    pub raw_response: T,
}

/// Trait for AI completion models that can generate text completions and handle streaming.
///
/// This trait provides a unified interface for different AI model providers,
/// allowing for easy switching between different models while maintaining
/// consistent behavior for completion requests and streaming responses.
pub trait CompletionModel: Clone + Send + Sync {
    /// The raw response type returned by the underlying completion model.
    type Response: Send + Sync;
    /// The raw response type returned by the underlying completion model when streaming.
    type StreamingResponse: Clone + Unpin + Send + Sync;

    /// Generates a completion response for the given completion request.
    ///
    /// # Arguments
    /// * `request` - The completion request containing messages, tools, and parameters
    ///
    /// # Returns
    /// * `Result<CompletionResponse<Self::Response>, CompletionError>` - The completion response or error
    fn completion(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<
        Output = Result<CompletionResponse<Self::Response>, CompletionError>,
    > + Send;

    /// Generates a streaming completion response for the given completion request.
    ///
    /// # Arguments
    /// * `request` - The completion request containing messages, tools, and parameters
    ///
    /// # Returns
    /// * `Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>` - The streaming response or error
    fn stream(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<
        Output = Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>,
    > + Send;

    /// Generates a completion request builder for the given messages.
    ///
    /// # Arguments
    /// * `prompt` - Vector of messages to start the conversation
    ///
    /// # Returns
    /// * `CompletionRequestBuilder<Self>` - A builder for constructing completion requests
    fn completion_request(&self, prompt: Vec<Message>) -> CompletionRequestBuilder<Self> {
        CompletionRequestBuilder::new(self.clone(), prompt)
    }
}

/// Builder for constructing completion requests with a fluent API.
///
/// `CompletionRequestBuilder` provides a convenient way to build completion requests
/// by chaining method calls. It supports adding messages, tools, and additional parameters.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompletionRequestBuilder<M: CompletionModel> {
    model: M,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
    additional_params: Option<serde_json::Value>,
}

impl<M: CompletionModel + Clone> CompletionRequestBuilder<M> {
    /// Creates a new completion request builder.
    ///
    /// # Arguments
    /// * `model` - The completion model to use
    /// * `messages` - Initial messages for the conversation
    ///
    /// # Returns
    /// * `CompletionRequestBuilder<M>` - A new builder instance
    pub fn new(model: M, messages: Vec<Message>) -> Self {
        Self {
            model,
            messages,
            tools: None,
            additional_params: None,
        }
    }

    /// Adds a message to the chat history for the completion request.
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Adds a list of messages to the chat history for the completion request.
    pub fn messages(self, messages: Vec<Message>) -> Self {
        messages
            .into_iter()
            .fold(self, |builder, msg| builder.message(msg))
    }

    /// Adds a tool to the completion request.
    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.as_mut().unwrap().push(tool);
        self
    }

    /// Adds a list of tools to the completion request.
    pub fn tools(self, tools: Vec<Tool>) -> Self {
        tools
            .into_iter()
            .fold(self, |builder, tool| builder.tool(tool))
    }

    /// Sets the additional parameters for the completion request.
    pub fn additional_params(mut self, additional_params: Option<serde_json::Value>) -> Self {
        self.additional_params = additional_params;
        self
    }

    /// Builds the completion request.
    pub fn build(self) -> CompletionRequest {
        CompletionRequest {
            messages: self.messages,
            tools: self.tools,
            additional_params: self.additional_params,
        }
    }

    /// Sends the completion request to the completion model provider and returns the completion response.
    pub async fn send(self) -> Result<CompletionResponse<M::Response>, CompletionError> {
        let model = self.model.clone();
        model.completion(self.build()).await
    }

    /// Stream the completion request
    pub async fn stream<'a>(
        self,
    ) -> Result<StreamingCompletionResponse<M::StreamingResponse>, CompletionError>
    where
        <M as CompletionModel>::StreamingResponse: 'a,
        Self: 'a,
    {
        let model = self.model.clone();
        model.stream(self.build()).await
    }
}
