use thiserror::Error;

/// workflow error types
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Node execution failed: {0}")]
    /// node execution failed
    NodeExecutionFailed(String),
    #[error("Channel not found: {0}")]
    /// channel not found
    ChannelNotFound(String),
    #[error("Invalid state: {0}")]
    /// invalid state
    InvalidState(String),
    #[error("Timeout: {0}")]
    /// operation timed out
    Timeout(String),
    #[error("Type mismatch: {0}")]
    /// type mismatch
    TypeMismatch(String),
    #[error("Configuration error: {0}")]
    /// configuration error
    ConfigError(String),
}

/// workflow result type
pub type WorkflowResult<T> = Result<T, WorkflowError>;
