//! Memory management system for AI conversations
//!
//! This module provides various memory implementations for storing and managing
//! conversation history in AI applications. It includes different memory types
//! optimized for different use cases and constraints.

pub mod buffer;
pub mod config;
pub mod error;
pub mod manager;
pub mod message;
pub mod token_buffer;
pub mod tokenizer;
pub mod window;

// Re-export main types
pub use crate::completions::messages::{Content, Message, Role};
pub use buffer::BufferMemory;
pub use config::{MemoryConfig, MemoryType};
pub use error::{MemoryError, MemoryResult};
pub use manager::MemoryManager;
pub use message::TimestampedMessage;
pub use token_buffer::TokenBufferMemory;
pub use tokenizer::{CharTokenizer, FixedTokenizer, SimpleTokenizer, Tokenizer};
pub use window::WindowMemory;

/// Base trait for memory systems
///
/// This trait defines the interface that all memory implementations must follow.
/// It provides methods for storing, retrieving, and managing conversation history.
pub trait Memory: Send + Sync {
    /// Add a message to memory
    fn add_message(&self, message: Message) -> MemoryResult<()>;

    /// Get all messages from memory
    fn get_messages(&self) -> MemoryResult<Vec<Message>>;

    /// Clear all messages from memory
    fn clear(&self) -> MemoryResult<()>;

    /// Get the number of messages in memory
    fn size(&self) -> MemoryResult<usize>;

    /// Get the last N messages from memory
    fn get_last_messages(&self, n: usize) -> MemoryResult<Vec<Message>>;

    /// Get the memory type name
    fn memory_type(&self) -> &'static str;

    /// Get an iterator over all messages, avoiding cloning
    fn iter_messages(&self) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>>;

    /// Get an iterator over the last N messages
    fn iter_last_messages(&self, n: usize) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>>;

    /// Check if memory is empty
    fn is_empty(&self) -> MemoryResult<bool> {
        Ok(self.size()? == 0)
    }

    /// Get memory statistics
    fn get_stats(&self) -> MemoryResult<MemoryStats> {
        let messages = self.get_messages()?;
        let mut user_count = 0;
        let mut assistant_count = 0;
        let mut system_count = 0;

        for message in &messages {
            match message.role {
                Role::User => user_count += 1,
                Role::Assistant => assistant_count += 1,
                Role::System => system_count += 1,
            }
        }

        Ok(MemoryStats {
            total_messages: messages.len(),
            user_messages: user_count,
            assistant_messages: assistant_count,
            system_messages: system_count,
            memory_type: self.memory_type().to_string(),
        })
    }
}

/// Memory statistics information
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total number of messages
    pub total_messages: usize,
    /// Number of user messages
    pub user_messages: usize,
    /// Number of assistant messages
    pub assistant_messages: usize,
    /// Number of system messages
    pub system_messages: usize,
    /// Type of memory implementation
    pub memory_type: String,
}

impl MemoryStats {
    /// Get the number of conversation turns (user messages)
    pub fn get_conversation_turns(&self) -> usize {
        self.user_messages
    }

    /// Get message distribution ratios (user, assistant, system)
    pub fn get_message_distribution(&self) -> (f64, f64, f64) {
        if self.total_messages == 0 {
            return (0.0, 0.0, 0.0);
        }

        let user_ratio = self.user_messages as f64 / self.total_messages as f64;
        let assistant_ratio = self.assistant_messages as f64 / self.total_messages as f64;
        let system_ratio = self.system_messages as f64 / self.total_messages as f64;

        (user_ratio, assistant_ratio, system_ratio)
    }
}
