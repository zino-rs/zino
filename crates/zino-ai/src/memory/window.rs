//! Window memory implementation
//!
//! This module provides a sliding window memory that keeps only the most recent N messages,
//! automatically removing older messages when the limit is exceeded.

use super::{Memory, MemoryResult, TimestampedMessage};
use crate::completions::messages::{Content, Message, Role};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Sliding window memory - keeps only the most recent N messages
///
/// This memory type maintains a fixed-size window of the most recent messages,
/// automatically removing older messages when new ones are added.
#[derive(Debug)]
pub struct WindowMemory {
    messages: Arc<RwLock<VecDeque<TimestampedMessage>>>,
    max_size: usize,
}

impl WindowMemory {
    /// Create a new window memory with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
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

    /// 获取窗口大小
    pub fn get_max_size(&self) -> usize {
        self.max_size
    }

    /// 设置窗口大小
    pub fn set_max_size(&mut self, max_size: usize) -> MemoryResult<()> {
        self.max_size = max_size;

        // 如果当前消息数量超过新的窗口大小，移除旧消息
        let mut messages = self.messages.write().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire write lock: {}", e))
        })?;

        while messages.len() > self.max_size {
            messages.pop_front();
        }
        Ok(())
    }

    /// 获取窗口使用率 (0.0 到 1.0)
    pub fn get_usage_ratio(&self) -> MemoryResult<f64> {
        let messages = self.messages.read().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(messages.len() as f64 / self.max_size as f64)
    }

    /// 检查窗口是否已满
    pub fn is_full(&self) -> MemoryResult<bool> {
        let messages = self.messages.read().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(messages.len() >= self.max_size)
    }
}

impl Memory for WindowMemory {
    fn add_message(&self, message: Message) -> MemoryResult<()> {
        let mut messages = self.messages.write().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire write lock: {}", e))
        })?;

        messages.push_back(TimestampedMessage::new(message));

        // 保持窗口大小
        while messages.len() > self.max_size {
            messages.pop_front();
        }
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
        "WindowMemory"
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
