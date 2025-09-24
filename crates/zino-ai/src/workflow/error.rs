use thiserror::Error;

/// Error types that can occur during workflow execution.
///
/// `WorkflowError` represents various failure modes that can happen
/// during workflow execution, providing detailed error messages for debugging.
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Node execution failed: {0}")]
    /// Node execution failed with the given error message.
    NodeExecutionFailed(String),
    #[error("Channel not found: {0}")]
    /// Channel with the given name was not found.
    ChannelNotFound(String),
    #[error("Invalid state: {0}")]
    /// Invalid state value or state transition.
    InvalidState(String),
    #[error("Timeout: {0}")]
    /// Operation timed out with the given message.
    Timeout(String),
    #[error("Type mismatch: {0}")]
    /// Type mismatch between expected and actual values.
    TypeMismatch(String),
    #[error("Configuration error: {0}")]
    /// Configuration error with the given message.
    ConfigError(String),
}

/// Result type for workflow operations.
///
/// `WorkflowResult<T>` is a type alias for `Result<T, WorkflowError>`,
/// providing a convenient way to handle workflow-specific errors.
pub type WorkflowResult<T> = Result<T, WorkflowError>;
