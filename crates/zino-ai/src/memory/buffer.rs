//! Buffer memory implementation
//!
//! This module provides a simple buffer-based memory that stores all conversation
//! history without any size limits or automatic cleanup.

use super::{Memory, MemoryResult, TimestampedMessage};
use crate::completions::messages::{Content, Message, Role};
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
        self.add_message(Message {
            role: Role::User,
            content: Content::Text(user_input),
        });
        self.add_message(Message {
            role: Role::Assistant,
            content: Content::Text(assistant_output),
        });
    }

    /// Get formatted conversation history as a string
    pub fn get_formatted_history(&self) -> MemoryResult<String> {
        let messages = self.get_messages()?;
        let mut formatted = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    formatted.push_str(&format!(
                        "System: {}\n",
                        message.content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Role::User => {
                    formatted.push_str(&format!(
                        "User: {}\n",
                        message.content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Role::Assistant => {
                    formatted.push_str(&format!(
                        "Assistant: {}\n",
                        message.content.as_text().unwrap_or(&"".to_string())
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
        let start = if len > n { len - n } else { 0 };
        Ok(messages
            .range(start..)
            .map(|tm| tm.message.clone())
            .collect())
    }

    fn memory_type(&self) -> &'static str {
        "BufferMemory"
    }

    fn iter_messages(&self) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>> {
        // 由于生命周期限制，我们返回一个收集的迭代器
        let messages = self.get_messages()?;
        Ok(Box::new(messages.into_iter()))
    }

    fn iter_last_messages(&self, n: usize) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>> {
        // 由于生命周期限制，我们返回一个收集的迭代器
        let messages = self.get_last_messages(n)?;
        Ok(Box::new(messages.into_iter()))
    }
}

impl Default for BufferMemory {
    fn default() -> Self {
        Self::new()
    }
}
