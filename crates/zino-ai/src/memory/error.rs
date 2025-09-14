use std::fmt;

/// 记忆系统错误类型
#[derive(Debug, Clone)]
pub enum MemoryError {
    /// 内存锁被污染
    LockPoisoned(String),
    /// Token限制超出
    TokenLimitExceeded { current: usize, max: usize },
    /// 消息数量限制超出
    MessageLimitExceeded { current: usize, max: usize },
    /// 序列化错误
    Serialization(String),
    /// 反序列化错误
    Deserialization(String),
    /// 配置错误
    Configuration(String),
    /// 存储错误
    Storage(String),
    /// 其他错误
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

impl From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, std::collections::VecDeque<crate::memory::TimestampedMessage>>>> for MemoryError {
    fn from(err: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, std::collections::VecDeque<crate::memory::TimestampedMessage>>>) -> Self {
        MemoryError::LockPoisoned(err.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, std::collections::VecDeque<crate::memory::TimestampedMessage>>>> for MemoryError {
    fn from(err: std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, std::collections::VecDeque<crate::memory::TimestampedMessage>>>) -> Self {
        MemoryError::LockPoisoned(err.to_string())
    }
}

/// 记忆系统结果类型
pub type MemoryResult<T> = Result<T, MemoryError>;

