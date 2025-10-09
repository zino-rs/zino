//! Memory manager implementation
//!
//! This module provides a unified memory manager that wraps different memory implementations
//! and provides a consistent interface for memory operations.

use super::{BufferMemory, TokenBufferMemory, WindowMemory};
use super::{Memory, MemoryConfig, MemoryResult};
use crate::completions::messages::Message;
use std::sync::Arc;

/// Memory manager - unified interface
///
/// This struct provides a unified interface for different memory implementations,
/// allowing easy switching between different memory types.
pub struct MemoryManager {
    memory: Arc<dyn Memory>,
}

impl std::fmt::Debug for MemoryManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryManager")
            .field("memory_type", &self.memory.memory_type())
            .field("size", &self.memory.size())
            .finish()
    }
}

impl MemoryManager {
    /// Create a manager with the specified memory instance
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }

    /// Create a manager with configuration
    pub fn with_config(config: MemoryConfig) -> MemoryResult<Self> {
        config.validate()?;

        let memory: Arc<dyn Memory> = match (config.max_tokens, config.max_messages) {
            (Some(max_tokens), _) => Arc::new(TokenBufferMemory::with_tokenizer(
                max_tokens,
                config.tokenizer,
            )),
            (_, Some(max_messages)) => Arc::new(WindowMemory::new(max_messages)),
            (None, None) => Arc::new(BufferMemory::new()),
        };

        Ok(Self { memory })
    }

    /// Create a buffer memory manager
    pub fn with_buffer_memory() -> Self {
        Self {
            memory: Arc::new(BufferMemory::new()),
        }
    }

    /// Create a sliding window memory manager
    pub fn with_window_memory(max_size: usize) -> Self {
        Self {
            memory: Arc::new(WindowMemory::new(max_size)),
        }
    }

    /// Create a token buffer memory manager
    pub fn with_token_buffer_memory(max_tokens: usize) -> Self {
        Self {
            memory: Arc::new(TokenBufferMemory::new(max_tokens)),
        }
    }

    /// Add a message to memory
    pub fn add_message(&self, message: Message) -> MemoryResult<()> {
        self.memory.add_message(message)
    }

    /// Add a conversation pair (user input and assistant output)
    pub fn add_conversation(
        &self,
        user_input: String,
        assistant_output: String,
    ) -> MemoryResult<()> {
        self.add_message(Message::user(user_input))?;
        self.add_message(Message::assistant(assistant_output))?;
        Ok(())
    }

    /// Get all messages from memory
    pub fn get_messages(&self) -> MemoryResult<Vec<Message>> {
        self.memory.get_messages()
    }

    /// Clear all messages from memory
    pub fn clear(&self) -> MemoryResult<()> {
        self.memory.clear()
    }

    /// Get the number of messages in memory
    pub fn size(&self) -> MemoryResult<usize> {
        self.memory.size()
    }

    /// Get the last N messages from memory
    pub fn get_last_messages(&self, n: usize) -> MemoryResult<Vec<Message>> {
        self.memory.get_last_messages(n)
    }

    /// Get memory type information
    pub fn get_memory_type(&self) -> String {
        self.memory.memory_type().to_string()
    }

    /// Check if memory is empty
    pub fn is_empty(&self) -> MemoryResult<bool> {
        self.memory.is_empty()
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryResult<super::MemoryStats> {
        self.memory.get_stats()
    }

    /// Export memory to JSON string
    pub fn export_to_json(&self) -> MemoryResult<String> {
        let messages = self.get_messages()?;
        serde_json::to_string_pretty(&messages)
            .map_err(|e| super::MemoryError::Serialization(e.to_string()))
    }

    /// Import memory from JSON string
    pub fn import_from_json(&self, json: &str) -> MemoryResult<()> {
        let messages: Vec<Message> = serde_json::from_str(json)
            .map_err(|e| super::MemoryError::Deserialization(e.to_string()))?;

        // Clear existing memory
        self.clear()?;

        // Add imported messages
        for message in messages {
            self.add_message(message)?;
        }

        Ok(())
    }

    /// Get formatted conversation history
    pub fn get_formatted_history(&self) -> MemoryResult<String> {
        let messages = self.get_messages()?;
        let mut formatted = String::new();

        for message in messages {
            match message {
                Message::System { content } => {
                    formatted.push_str(&format!(
                        "System: {}\n",
                        content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Message::User { content } => {
                    formatted.push_str(&format!(
                        "User: {}\n",
                        content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Message::Assistant { content, .. } => {
                    formatted.push_str(&format!(
                        "Assistant: {}\n",
                        content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Message::Tool { content, .. } => {
                    formatted.push_str(&format!(
                        "Tool: {}\n",
                        content.as_text().unwrap_or(&"".to_string())
                    ));
                }
            }
        }

        Ok(formatted)
    }
}

impl MemoryManager {
    /// Add a system message to the conversation
    ///
    /// # Arguments
    /// * `system_message` - The system message content
    ///
    /// # Returns
    /// * `Result<(), MemoryError>` - Success or error
    pub fn add_system_message(&self, system_message: &str) -> Result<(), super::MemoryError> {
        let message = Message::system(system_message.to_string());
        self.memory.add_message(message)
    }

    /// Add a tool result message to the conversation
    ///
    /// # Arguments
    /// * `tool_call_id` - The ID of the tool call
    /// * `result` - The result from the tool
    ///
    /// # Returns
    /// * `Result<(), MemoryError>` - Success or error
    pub fn add_tool_result(
        &self,
        tool_call_id: String,
        result: String,
    ) -> Result<(), super::MemoryError> {
        let message = Message::tool(result, tool_call_id);
        self.memory.add_message(message)
    }

    /// Add a user message with multimodal content
    ///
    /// # Arguments
    /// * `content` - The multimodal content from the user
    ///
    /// # Returns
    /// * `Result<(), MemoryError>` - Success or error
    pub fn add_user_multimodal(
        &self,
        content: crate::completions::content::Content,
    ) -> Result<(), super::MemoryError> {
        let message = Message::user_multimodal(content);
        self.memory.add_message(message)
    }

    /// Add an assistant message with tool calls
    ///
    /// # Arguments
    /// * `text` - The assistant message text
    /// * `tool_calls` - Vector of tool calls
    ///
    /// # Returns
    /// * `Result<(), MemoryError>` - Success or error
    pub fn add_assistant_with_tools(
        &self,
        text: String,
        tool_calls: Vec<crate::completions::messages::ToolCall>,
    ) -> Result<(), super::MemoryError> {
        let message = Message::assistant_with_tool_calls(text, tool_calls);
        self.memory.add_message(message)
    }

    /// Get conversation statistics as a formatted string
    ///
    /// # Returns
    /// * `Result<String, MemoryError>` - Formatted statistics
    pub fn get_conversation_summary(&self) -> Result<String, super::MemoryError> {
        let stats = self.memory.get_stats()?;
        Ok(format!(
            "Conversation stats: {} total messages, {} user, {} assistant, {} system, {} tool",
            stats.total_messages,
            stats.user_messages,
            stats.assistant_messages,
            stats.system_messages,
            stats.tool_messages
        ))
    }

    /// Get memory usage summary as a formatted string
    ///
    /// # Returns
    /// * `Result<String, MemoryError>` - Formatted memory usage
    pub fn get_memory_summary(&self) -> Result<String, super::MemoryError> {
        let stats = self.memory.get_stats()?;
        let (user_ratio, assistant_ratio, system_ratio, tool_ratio) =
            stats.get_message_distribution();
        Ok(format!(
            "Memory usage: {:.1}% user, {:.1}% assistant, {:.1}% system, {:.1}% tool",
            user_ratio * 100.0,
            assistant_ratio * 100.0,
            system_ratio * 100.0,
            tool_ratio * 100.0
        ))
    }
}
