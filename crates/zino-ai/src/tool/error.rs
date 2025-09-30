//! Error types for the tool execution system.

use std::fmt;

/// Error types for function execution.
#[derive(Debug, Clone)]
pub enum FunctionError {
    /// Function not found in registry
    FunctionNotFound(String),
    /// Invalid function arguments
    InvalidArguments(String),
    /// Function execution failed
    ExecutionFailed(String),
    /// JSON serialization/deserialization error
    JsonError(String),
    /// Timeout during function execution
    Timeout,
    /// Permission denied for function execution
    PermissionDenied(String),
    /// Function is not available
    FunctionUnavailable(String),
    /// Other error
    Other(String),
}

impl fmt::Display for FunctionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionError::FunctionNotFound(name) => {
                write!(f, "Function '{}' not found in registry", name)
            }
            FunctionError::InvalidArguments(msg) => {
                write!(f, "Invalid arguments: {}", msg)
            }
            FunctionError::ExecutionFailed(msg) => {
                write!(f, "Function execution failed: {}", msg)
            }
            FunctionError::JsonError(msg) => {
                write!(f, "JSON error: {}", msg)
            }
            FunctionError::Timeout => {
                write!(f, "Function execution timeout")
            }
            FunctionError::PermissionDenied(msg) => {
                write!(f, "Permission denied: {}", msg)
            }
            FunctionError::FunctionUnavailable(msg) => {
                write!(f, "Function unavailable: {}", msg)
            }
            FunctionError::Other(msg) => {
                write!(f, "Error: {}", msg)
            }
        }
    }
}

impl std::error::Error for FunctionError {}

impl From<serde_json::Error> for FunctionError {
    fn from(err: serde_json::Error) -> Self {
        FunctionError::JsonError(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for FunctionError {
    fn from(err: std::time::SystemTimeError) -> Self {
        FunctionError::Other(format!("System time error: {}", err))
    }
}

/// Result type for function execution.
pub type FunctionResult<T> = Result<T, FunctionError>;
