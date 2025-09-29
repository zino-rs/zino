//! Buffer memory implementation
//!
//! This module provides a simple buffer-based memory that stores all conversation
//! history without any size limits or automatic cleanup.

use super::{Memory, MemoryResult, TimestampedMessage};
use crate::completions::messages::Message;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Basic buffer memory - stores all conversation history
///
/// This memory type keeps all messages in memory without any size constraints.
/// It's suitable for short conversations or when you need to preserve all history.
#[derive(Debug)]
pub struct BufferMemory {
    messages: Arc<RwLock<VecDeque<TimestampedMessage>>>,
}

impl BufferMemory {
    /// Create a new buffer memory instance
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Add a conversation pair (user input and assistant output)
    pub fn add_conversation(&self, user_input: String, assistant_output: String) {
        let _ = self.add_message(Message::user(user_input));
        let _ = self.add_message(Message::assistant(assistant_output));
    }

    /// Get formatted conversation history as a string
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

impl Memory for BufferMemory {
    fn add_message(&self, message: Message) -> MemoryResult<()> {
        let mut messages = self.messages.write().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire write lock: {}", e))
        })?;
        messages.push_back(TimestampedMessage::new(message));
        Ok(())
    }

    fn get_messages(&self) -> MemoryResult<Vec<Message>> {
        let messages = self.messages.read().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(messages.iter().map(|tm| tm.message.clone()).collect())
    }

    fn clear(&self) -> MemoryResult<()> {
        let mut messages = self.messages.write().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire write lock: {}", e))
        })?;
        messages.clear();
        Ok(())
    }

    fn size(&self) -> MemoryResult<usize> {
        let messages = self.messages.read().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(messages.len())
    }

    fn get_last_messages(&self, n: usize) -> MemoryResult<Vec<Message>> {
        let messages = self.messages.read().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire read lock: {}", e))
        })?;
        let len = messages.len();
        let start = len.saturating_sub(n);
        Ok(messages
            .range(start..)
            .map(|tm| tm.message.clone())
            .collect())
    }

    fn memory_type(&self) -> &'static str {
        "BufferMemory"
    }

    fn iter_messages(&self) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>> {
        // Due to lifetime constraints, we return a collected iterator
        let messages = self.get_messages()?;
        Ok(Box::new(messages.into_iter()))
    }

    fn iter_last_messages(&self, n: usize) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>> {
        // Due to lifetime constraints, we return a collected iterator
        let messages = self.get_last_messages(n)?;
        Ok(Box::new(messages.into_iter()))
    }
}

impl Default for BufferMemory {
    fn default() -> Self {
        Self::new()
    }
}
