use thiserror::Error;

/// 工作流错误类型
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Node execution failed: {0}")]
    NodeExecutionFailed(String),
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// 结果类型
pub type WorkflowResult<T> = Result<T, WorkflowError>;
