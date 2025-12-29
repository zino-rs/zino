//! Memory configuration and types
//!
//! This module defines configuration structures and memory types for the memory system.

use crate::memory::error::{MemoryError, MemoryResult};
use crate::memory::tokenizer::{SimpleTokenizer, Tokenizer};
use std::time::Duration;

/// Memory system configuration
#[derive(Debug)]
pub struct MemoryConfig {
    /// Maximum number of tokens (None means unlimited)
    pub max_tokens: Option<usize>,
    /// Maximum number of messages (None means unlimited)
    pub max_messages: Option<usize>,
    /// Whether to enable auto-save
    pub auto_save: bool,
    /// Auto-save interval
    pub save_interval: Duration,
    /// Tokenizer instance
    pub tokenizer: Box<dyn Tokenizer>,
    /// Whether to enable compression
    pub enable_compression: bool,
    /// Whether to enable statistics
    pub enable_stats: bool,
}

impl MemoryConfig {
    /// Create default configuration
    pub fn new_default() -> Self {
        Self {
            max_tokens: None,
            max_messages: None,
            auto_save: false,
            save_interval: Duration::from_secs(300), // 5 minutes
            tokenizer: Box::new(SimpleTokenizer::new()),
            enable_compression: false,
            enable_stats: true,
        }
    }

    /// Create buffer memory configuration
    pub fn buffer() -> Self {
        Self::new_default()
    }

    /// Create window memory configuration
    pub fn window(max_messages: usize) -> Self {
        Self {
            max_messages: Some(max_messages),
            ..Self::new_default()
        }
    }

    /// Create token buffer memory configuration
    pub fn token_buffer(max_tokens: usize) -> Self {
        Self {
            max_tokens: Some(max_tokens),
            ..Self::new_default()
        }
    }

    /// Set maximum token count
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set maximum message count
    pub fn with_max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = Some(max_messages);
        self
    }

    /// Set tokenizer
    pub fn with_tokenizer<T: Tokenizer + 'static>(mut self, tokenizer: T) -> Self {
        self.tokenizer = Box::new(tokenizer);
        self
    }

    /// Enable auto-save
    pub fn with_auto_save(mut self, interval: Duration) -> Self {
        self.auto_save = true;
        self.save_interval = interval;
        self
    }

    /// Enable compression
    pub fn with_compression(mut self) -> Self {
        self.enable_compression = true;
        self
    }

    /// Disable statistics
    pub fn without_stats(mut self) -> Self {
        self.enable_stats = false;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> MemoryResult<()> {
        if let Some(max_tokens) = self.max_tokens
            && max_tokens == 0
        {
            return Err(MemoryError::Configuration(
                "max_tokens cannot be zero".to_string(),
            ));
        }

        if let Some(max_messages) = self.max_messages
            && max_messages == 0
        {
            return Err(MemoryError::Configuration(
                "max_messages cannot be zero".to_string(),
            ));
        }

        if self.auto_save && self.save_interval.is_zero() {
            return Err(MemoryError::Configuration(
                "save_interval cannot be zero when auto_save is enabled".to_string(),
            ));
        }

        Ok(())
    }

    /// Get configuration summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(max_tokens) = self.max_tokens {
            parts.push(format!("max_tokens={}", max_tokens));
        }

        if let Some(max_messages) = self.max_messages {
            parts.push(format!("max_messages={}", max_messages));
        }

        if self.auto_save {
            parts.push(format!("auto_save={}s", self.save_interval.as_secs()));
        }

        parts.push(format!("tokenizer={}", self.tokenizer.name()));
        parts.push(format!("compression={}", self.enable_compression));
        parts.push(format!("stats={}", self.enable_stats));

        parts.join(", ")
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self::new_default()
    }
}

/// Memory type enumeration (for backward compatibility)
#[derive(Debug, Clone)]
pub enum MemoryType {
    /// Buffer memory - stores all messages
    Buffer,
    /// Window memory - stores only the last N messages
    Window(usize),
    /// Token buffer memory - stores messages up to a token limit
    TokenBuffer(usize),
}

impl MemoryType {
    /// Convert to configuration
    pub fn to_config(&self) -> MemoryConfig {
        match self {
            MemoryType::Buffer => MemoryConfig::buffer(),
            MemoryType::Window(size) => MemoryConfig::window(*size),
            MemoryType::TokenBuffer(tokens) => MemoryConfig::token_buffer(*tokens),
        }
    }
}
