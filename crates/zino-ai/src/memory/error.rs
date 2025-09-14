//! Memory system error types
//!
//! This module defines error types used throughout the memory system.

use std::fmt;

/// Memory system error types
#[derive(Debug, Clone)]
pub enum MemoryError {
    /// Memory lock was poisoned
    LockPoisoned(String),
    /// Token limit exceeded
    TokenLimitExceeded { current: usize, max: usize },
    /// Message count limit exceeded
    MessageLimitExceeded { current: usize, max: usize },
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Configuration error
    Configuration(String),
    /// Storage error
    Storage(String),
    /// Other error
    Other(String),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::LockPoisoned(msg) => write!(f, "Memory lock poisoned: {}", msg),
            MemoryError::TokenLimitExceeded { current, max } => {
                write!(f, "Token limit exceeded: {} > {}", current, max)
            }
            MemoryError::MessageLimitExceeded { current, max } => {
                write!(f, "Message limit exceeded: {} > {}", current, max)
            }
            MemoryError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            MemoryError::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            MemoryError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            MemoryError::Storage(msg) => write!(f, "Storage error: {}", msg),
            MemoryError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MemoryError {}

impl From<serde_json::Error> for MemoryError {
    fn from(err: serde_json::Error) -> Self {
        MemoryError::Serialization(err.to_string())
    }
}

impl
    From<
        std::sync::PoisonError<
            std::sync::RwLockReadGuard<
                '_,
                std::collections::VecDeque<crate::memory::TimestampedMessage>,
            >,
        >,
    > for MemoryError
{
    fn from(
        err: std::sync::PoisonError<
            std::sync::RwLockReadGuard<
                '_,
                std::collections::VecDeque<crate::memory::TimestampedMessage>,
            >,
        >,
    ) -> Self {
        MemoryError::LockPoisoned(err.to_string())
    }
}

impl
    From<
        std::sync::PoisonError<
            std::sync::RwLockWriteGuard<
                '_,
                std::collections::VecDeque<crate::memory::TimestampedMessage>,
            >,
        >,
    > for MemoryError
{
    fn from(
        err: std::sync::PoisonError<
            std::sync::RwLockWriteGuard<
                '_,
                std::collections::VecDeque<crate::memory::TimestampedMessage>,
            >,
        >,
    ) -> Self {
        MemoryError::LockPoisoned(err.to_string())
    }
}

/// Memory system result type
pub type MemoryResult<T> = Result<T, MemoryError>;
