//! Error types for vector operations.
//!
//! This module defines error types and result aliases for
//! vector embedding operations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for vector operations.
pub type VectorResult<T> = Result<T, VectorError>;

/// Errors that can occur in vector operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorError {
    /// Embedding model error
    EmbeddingError(String),
    /// I/O error
    IoError(String),
}

impl fmt::Display for VectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VectorError::EmbeddingError(msg) => {
                write!(f, "Embedding error: {}", msg)
            }
            VectorError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
        }
    }
}

impl std::error::Error for VectorError {}

impl From<std::io::Error> for VectorError {
    fn from(err: std::io::Error) -> Self {
        VectorError::IoError(err.to_string())
    }
}

impl From<crate::embeddings::embedding::EmbeddingError> for VectorError {
    fn from(err: crate::embeddings::embedding::EmbeddingError) -> Self {
        VectorError::EmbeddingError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for VectorError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        VectorError::IoError(err.to_string())
    }
}
