//! Universal embedding model implementations.
//!
//! This module provides a unified interface for embedding text and messages
//! across different AI service providers.

use crate::completions::messages::Message;
use crate::embeddings::error::VectorError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during embedding operations.
///
/// This enum represents various failure modes that can happen
/// during embedding text or message processing.
#[derive(Debug, Error)]
pub enum EmbeddingError {
    /// HTTP request failed with the given error.
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed.
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Vector operation failed with the given error.
    #[error("VectorError: {0}")]
    VectorError(#[from] VectorError),

    /// Invalid response received from the embedding service.
    #[error("InvalidResponse: {0}")]
    InvalidResponse(String),
}

/// Universal embedding response structure
///
/// This generic structure allows users to handle response details themselves
/// by providing the raw response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding<T> {
    /// Raw response data from the embedding service
    pub raw_response: T,
}

/// Universal embedding model client
///
/// This struct provides a unified interface for embedding text across different
/// AI service providers. It abstracts away provider-specific differences.
#[derive(Debug, Clone)]
pub struct EmbeddingModel {
    /// API endpoint URL
    pub url: String,
    /// API key for authentication
    pub api_key: String,
}

impl EmbeddingModel {
    /// Create a new embedding model client
    ///
    /// # Arguments
    /// * `url` - The API endpoint URL for the embedding service
    /// * `api_key` - The API key for authentication
    ///
    /// ```
    pub fn new(url: String, api_key: String) -> Self {
        Self { url, api_key }
    }

    /// Send embedding request with custom parameters
    ///
    /// # Arguments
    /// * `request_body` - Complete request parameters as JSON
    ///
    /// # Returns
    /// * `Result<Embedding<serde_json::Value>, EmbeddingError>` - The embedding result
    pub async fn embed(
        &self,
        request_body: serde_json::Value,
    ) -> Result<Embedding<serde_json::Value>, EmbeddingError> {
        self.make_request(request_body).await
    }

    /// Embed a Message directly
    ///
    /// # Arguments
    /// * `message` - The message to embed
    /// * `model_name` - The embedding model name to use
    ///
    /// # Returns
    /// * `Result<Embedding<serde_json::Value>, EmbeddingError>` - The embedding result
    ///
    pub async fn embed_message(
        &self,
        message: &Message,
        model_name: &str,
    ) -> Result<Embedding<serde_json::Value>, EmbeddingError> {
        // Extract text content from the message
        let text_content = self.extract_text_from_message(message);

        let request_body = serde_json::json!({
            "input": text_content,
            "model": model_name
        });

        self.make_request(request_body).await
    }

    /// Extract text content from a Message
    ///
    /// # Arguments
    /// * `message` - The message to extract text from
    ///
    /// # Returns
    /// * `String` - The extracted text content
    fn extract_text_from_message(&self, message: &Message) -> String {
        match message {
            Message::System { content } => content.as_string(),
            Message::User { content } => content.as_string(),
            Message::Assistant { content, .. } => content.as_string(),
            Message::Tool { content, .. } => content.as_string(),
        }
    }

    /// Make the actual HTTP request to the embedding service
    ///
    /// # Arguments
    /// * `request_body` - The JSON request body
    ///
    /// # Returns
    /// * `Result<Embedding<serde_json::Value>, EmbeddingError>` - The embedding response
    async fn make_request(
        &self,
        request_body: serde_json::Value,
    ) -> Result<Embedding<serde_json::Value>, EmbeddingError> {
        let client = Client::new();
        let response = client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(EmbeddingError::InvalidResponse(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await?;
        let parsed_response: serde_json::Value = serde_json::from_str(&response_text)?;

        Ok(Embedding {
            raw_response: parsed_response,
        })
    }
}
