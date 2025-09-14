//! Token buffer memory implementation
//!
//! This module provides a memory implementation that limits the total number of tokens
//! stored, automatically removing older messages when the token limit is exceeded.

use super::{Memory, MemoryResult, SimpleTokenizer, TimestampedMessage, Tokenizer};
use crate::completions::messages::{Content, Message};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Token-based memory - limits total token count
///
/// This memory type automatically manages message storage based on token count,
/// removing older messages when the token limit is exceeded.
#[derive(Debug)]
pub struct TokenBufferMemory {
    messages: Arc<RwLock<VecDeque<TimestampedMessage>>>,
    max_tokens: usize,
    tokenizer: Box<dyn Tokenizer>,
}

impl TokenBufferMemory {
    /// Create a new token buffer memory with default tokenizer
    pub fn new(max_tokens: usize) -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
            max_tokens,
            tokenizer: Box::new(SimpleTokenizer::new()),
        }
    }

    /// Create a new token buffer memory with custom tokenizer
    pub fn with_tokenizer(max_tokens: usize, tokenizer: Box<dyn Tokenizer>) -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
            max_tokens,
            tokenizer,
        }
    }

    /// 计算文本的token数量
    pub fn count_tokens(&self, text: &str) -> MemoryResult<usize> {
        self.tokenizer.count_tokens(text)
    }

    /// 计算当前总 token 数
    pub fn total_tokens(&self) -> MemoryResult<usize> {
        let messages = self.messages.read().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire read lock: {}", e))
        })?;

        self.calculate_tokens_with_messages(&messages)
    }

    /// 计算给定消息列表的token数量（不需要获取锁）
    fn calculate_tokens_with_messages(
        &self,
        messages: &std::collections::VecDeque<TimestampedMessage>,
    ) -> MemoryResult<usize> {
        let mut total = 0;
        for tm in messages.iter() {
            match &tm.message.content {
                Content::Text(text) => {
                    total += self.count_tokens(text)?;
                }
                Content::Multimodal(_) => {
                    total += 100; // 假设多模态内容为 100 tokens
                }
            }
        }
        Ok(total)
    }

    /// 获取最大 token 限制
    pub fn get_max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// 设置最大 token 限制
    pub fn set_max_tokens(&mut self, max_tokens: usize) -> MemoryResult<()> {
        self.max_tokens = max_tokens;

        // 如果当前 token 数量超过新的限制，移除旧消息
        let mut messages = self.messages.write().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire write lock: {}", e))
        })?;

        while self.total_tokens()? > self.max_tokens && !messages.is_empty() {
            messages.pop_front();
        }
        Ok(())
    }

    /// 获取 token 使用率 (0.0 到 1.0)
    pub fn get_token_usage_ratio(&self) -> MemoryResult<f64> {
        let current_tokens = self.total_tokens()?;
        Ok(current_tokens as f64 / self.max_tokens as f64)
    }

    /// 检查是否接近 token 限制
    pub fn is_near_limit(&self, threshold: f64) -> MemoryResult<bool> {
        Ok(self.get_token_usage_ratio()? >= threshold)
    }

    /// 获取剩余 token 容量
    pub fn get_remaining_tokens(&self) -> MemoryResult<usize> {
        let current_tokens = self.total_tokens()?;
        Ok(self.max_tokens.saturating_sub(current_tokens))
    }
}

impl Memory for TokenBufferMemory {
    fn add_message(&self, message: Message) -> MemoryResult<()> {
        let mut messages = self.messages.write().map_err(|e| {
            super::MemoryError::LockPoisoned(format!("Failed to acquire write lock: {}", e))
        })?;

        messages.push_back(TimestampedMessage::new(message));

        // 移除旧消息直到 token 数量在限制内
        // 直接在写锁内计算token数量，避免死锁
        while self.calculate_tokens_with_messages(&messages)? > self.max_tokens
            && !messages.is_empty()
        {
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
        "TokenBufferMemory"
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
