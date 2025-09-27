//! Message types and conversation handling for AI applications.
//!
//! This module provides comprehensive support for AI conversation messages,
//! including system, user, assistant, and tool messages with multimodal content support.

use crate::completions::content::Content;
use serde::{Deserialize, Serialize};

/// Message types for AI conversations.
///
/// `Message` represents different types of messages in a conversation,
/// supporting both simple text and multimodal content.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// System message with content.
    System {
        /// The content of the system message.
        content: Content,
    },

    /// User message with content.
    User {
        /// The content of the user message.
        content: Content,
    },

    /// Assistant message with content and optional tool calls.
    Assistant {
        /// The content of the assistant message.
        /// This can be either text content or tool-related content.
        content: Content,
        /// Optional tool calls that the assistant wants to make.
        /// This is used when the assistant needs to call external functions or tools.
        tool_calls: Option<Vec<ToolCall>>,
    },

    /// Tool message with content and tool call ID.
    Tool {
        /// The result content of the tool call.
        content: Content,
        /// The tool call ID that this message corresponds to.
        tool_call_id: String,
    },
}

impl Message {
    /// Creates a new system message with text content.
    ///
    /// # Arguments
    /// * `text` - The system message text
    ///
    /// # Returns
    /// * `Message::System` - The system message
    pub fn system(text: String) -> Self {
        Message::System {
            content: Content::text(text),
        }
    }

    /// Creates a new user message with text content.
    ///
    /// # Arguments
    /// * `text` - The user message text
    ///
    /// # Returns
    /// * `Message::User` - The user message
    pub fn user(text: String) -> Self {
        Message::User {
            content: Content::text(text),
        }
    }

    /// Creates a new user message with multimodal content.
    ///
    /// # Arguments
    /// * `content` - The multimodal content
    ///
    /// # Returns
    /// * `Message::User` - The user message
    pub fn user_multimodal(content: Content) -> Self {
        Message::User { content }
    }

    /// Creates a new assistant message with text content.
    ///
    /// # Arguments
    /// * `text` - The assistant message text
    ///
    /// # Returns
    /// * `Message::Assistant` - The assistant message
    pub fn assistant(text: String) -> Self {
        Message::Assistant {
            content: Content::text(text),
            tool_calls: None,
        }
    }

    /// Creates a new assistant message with text content and tool calls.
    ///
    /// # Arguments
    /// * `text` - The assistant message text
    /// * `tool_calls` - Vector of tool calls that the assistant wants to make
    ///
    /// # Returns
    /// * `Message::Assistant` - The assistant message with tool calls
    pub fn assistant_with_tool_calls(text: String, tool_calls: Vec<ToolCall>) -> Self {
        Message::Assistant {
            content: Content::text(text),
            tool_calls: Some(tool_calls),
        }
    }

    /// Creates a new assistant message with multimodal content and tool calls.
    ///
    /// # Arguments
    /// * `content` - The multimodal content
    /// * `tool_calls` - Vector of tool calls that the assistant wants to make
    ///
    /// # Returns
    /// * `Message::Assistant` - The assistant message with tool calls
    pub fn assistant_multimodal_with_tool_calls(
        content: Content,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Message::Assistant {
            content,
            tool_calls: Some(tool_calls),
        }
    }

    /// Creates a new assistant message with text content and tool calls from JSON.
    ///
    /// # Arguments
    /// * `text` - The assistant message text
    /// * `tool_calls_json` - JSON string containing tool calls array
    ///
    /// # Returns
    /// * `Result<Message, serde_json::Error>` - The assistant message with tool calls or error
    pub fn assistant_with_tool_calls_json(
        text: String,
        tool_calls_json: &str,
    ) -> Result<Self, serde_json::Error> {
        let tool_calls: Vec<ToolCall> = serde_json::from_str(tool_calls_json)?;
        Ok(Message::Assistant {
            content: Content::text(text),
            tool_calls: Some(tool_calls),
        })
    }

    /// Creates a new assistant message with text content and tool calls from JSON value.
    ///
    /// # Arguments
    /// * `text` - The assistant message text
    /// * `tool_calls_value` - JSON value containing tool calls array
    ///
    /// # Returns
    /// * `Result<Message, serde_json::Error>` - The assistant message with tool calls or error
    pub fn assistant_with_tool_calls_value(
        text: String,
        tool_calls_value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        let tool_calls: Vec<ToolCall> = serde_json::from_value(tool_calls_value)?;
        Ok(Message::Assistant {
            content: Content::text(text),
            tool_calls: Some(tool_calls),
        })
    }

    /// Creates a new tool message with text content and tool call ID.
    ///
    /// # Arguments
    /// * `text` - The tool message text
    /// * `tool_call_id` - The tool call ID that this message corresponds to
    ///
    /// # Returns
    /// * `Message::Tool` - The tool message
    pub fn tool(text: String, tool_call_id: String) -> Self {
        Message::Tool {
            content: Content::text(text),
            tool_call_id,
        }
    }

    /// Creates a new tool message with multimodal content and tool call ID.
    ///
    /// # Arguments
    /// * `content` - The multimodal content
    /// * `tool_call_id` - The tool call ID that this message corresponds to
    ///
    /// # Returns
    /// * `Message::Tool` - The tool message
    pub fn tool_multimodal(content: Content, tool_call_id: String) -> Self {
        Message::Tool {
            content,
            tool_call_id,
        }
    }

    /// Gets the content of the message.
    ///
    /// # Returns
    /// * `&Content` - The message content
    pub fn content(&self) -> &Content {
        match self {
            Message::System { content } => content,
            Message::User { content } => content,
            Message::Assistant { content, .. } => content,
            Message::Tool { content, .. } => content,
        }
    }

    /// Gets the role of the message as a string.
    ///
    /// # Returns
    /// * `&str` - The role string
    pub fn role(&self) -> &'static str {
        match self {
            Message::System { .. } => "system",
            Message::User { .. } => "user",
            Message::Assistant { .. } => "assistant",
            Message::Tool { .. } => "tool",
        }
    }

    /// Gets the tool call ID if this is a tool message.
    ///
    /// # Returns
    /// * `Some(&String)` - If this is a tool message
    /// * `None` - If this is not a tool message
    pub fn tool_call_id(&self) -> Option<&String> {
        match self {
            Message::Tool { tool_call_id, .. } => Some(tool_call_id),
            _ => None,
        }
    }

    /// Gets the tool calls if this is an assistant message with tool calls.
    ///
    /// # Returns
    /// * `Some(&Vec<ToolCall>)` - If this is an assistant message with tool calls
    /// * `None` - If this is not an assistant message or has no tool calls
    pub fn tool_calls(&self) -> Option<&Vec<ToolCall>> {
        match self {
            Message::Assistant { tool_calls, .. } => tool_calls.as_ref(),
            _ => None,
        }
    }

    /// Converts the message to a string representation.
    ///
    /// # Returns
    /// * `String` - The string representation of the message
    pub fn as_string(&self) -> String {
        format!("{}: {}", self.role(), self.content().as_string())
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Tool call structure for assistant messages.
///
/// `ToolCall` represents a tool call that an assistant wants to make.
/// This is typically included in assistant messages when the assistant
/// needs to call external functions or tools to complete a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    /// This ID is used to match the tool call with its corresponding tool message.
    pub id: String,
    /// The type of tool call, typically "function".
    #[serde(rename = "type")]
    pub r#type: String,
    /// The function to be called.
    pub function: FunctionResult,
    /// Optional MCP (Model Context Protocol) data.
    pub mcp: Option<serde_json::Value>,
}

/// Function structure for tool calls.
///
/// `FunctionResult` represents a function that can be called by the assistant.
/// It contains the function name and its arguments as a JSON value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionResult {
    /// The name of the function to be called.
    pub name: String,
    /// The arguments for the function as a JSON value.
    #[serde(rename = "arguments")]
    pub arguments: serde_json::Value,
}

impl ToolCall {
    /// Creates a new tool call with the specified parameters.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this tool call
    /// * `function_name` - Name of the function to call
    /// * `arguments` - Function arguments as a JSON value
    ///
    /// # Returns
    /// * `ToolCall` - The created tool call
    pub fn new(id: String, function_name: String, arguments: serde_json::Value) -> Self {
        Self {
            id,
            r#type: "function".to_string(),
            function: FunctionResult {
                name: function_name,
                arguments,
            },
            mcp: None,
        }
    }

    /// Creates a new tool call from JSON string.
    ///
    /// # Arguments
    /// * `tool_call_json` - JSON string containing tool call data
    ///
    /// # Returns
    /// * `Result<ToolCall, serde_json::Error>` - The parsed tool call or error
    pub fn from_json(tool_call_json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(tool_call_json)
    }

    /// Creates a new tool call from JSON value.
    ///
    /// # Arguments
    /// * `tool_call_value` - JSON value containing tool call data
    ///
    /// # Returns
    /// * `Result<ToolCall, serde_json::Error>` - The parsed tool call or error
    pub fn from_value(tool_call_value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(tool_call_value)
    }
}

impl FunctionResult {
    /// Creates a new function result with the specified parameters.
    ///
    /// # Arguments
    /// * `name` - Name of the function
    /// * `arguments` - Function arguments as a JSON value
    ///
    /// # Returns
    /// * `FunctionResult` - The created function result
    pub fn new(name: String, arguments: serde_json::Value) -> Self {
        Self { name, arguments }
    }

    /// Creates a new function result from JSON string arguments.
    ///
    /// # Arguments
    /// * `name` - Name of the function
    /// * `arguments_json` - Function arguments as a JSON string
    ///
    /// # Returns
    /// * `Result<FunctionResult, serde_json::Error>` - The created function result or error
    pub fn from_json_args(name: String, arguments_json: &str) -> Result<Self, serde_json::Error> {
        let arguments: serde_json::Value = serde_json::from_str(arguments_json)?;
        Ok(Self { name, arguments })
    }
}
