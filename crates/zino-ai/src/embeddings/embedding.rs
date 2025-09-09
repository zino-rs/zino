//! The module defines the [EmbeddingModel] trait, which represents an embedding model that can
//! generate embeddings for documents.
//!
//! The module also defines the [Embedding] struct, which represents a single document embedding.
//!
//! Finally, the module defines the [EmbeddingError] enum, which represents various errors that
//! can occur during embedding generation or processing.

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    /// Http error (e.g.: connection error, timeout, etc.)
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Json error (e.g.: serialization, deserialization)
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error processing the document for embedding
    #[error("DocumentError: {0}")]
    DocumentError(Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Error parsing the completion response
    #[error("ResponseError: {0}")]
    ResponseError(String),

    /// Error returned by the embedding model provider
    #[error("ProviderError: {0}")]
    ProviderError(String),
}

/// Trait for embedding models that can generate embeddings for documents.
pub trait EmbeddingModel: Clone + Sync + Send {
    /// The maximum number of documents that can be embedded in a single request.
    const MAX_DOCUMENTS: usize;

    /// The number of dimensions in the embedding vector.
    fn ndims(&self) -> usize;

    /// Embed multiple text documents in a single request
    fn embed_texts(
        &self,
        texts: impl IntoIterator<Item = String> + Send,
    ) -> impl std::future::Future<Output = Result<Vec<Embedding>, EmbeddingError>> + Send;

    /// Embed a single text document.
    fn embed_text(
        &self,
        text: &str,
    ) -> impl std::future::Future<Output = Result<Embedding, EmbeddingError>> + Send {
        async {
            Ok(self
                .embed_texts(vec![text.to_string()])
                .await?
                .pop()
                .expect("There should be at least one embedding"))
        }
    }
}

pub trait EmbeddingModelDyn: Sync + Send {
    fn max_documents(&self) -> usize;
    fn ndims(&self) -> usize;
    fn embed_text<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Embedding, EmbeddingError>>;
    fn embed_texts(
        &self,
        texts: Vec<String>,
    ) -> BoxFuture<'_, Result<Vec<Embedding>, EmbeddingError>>;
}

impl<T: EmbeddingModel> EmbeddingModelDyn for T {
    fn max_documents(&self) -> usize {
        T::MAX_DOCUMENTS
    }

    fn ndims(&self) -> usize {
        self.ndims()
    }

    fn embed_text<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Embedding, EmbeddingError>> {
        Box::pin(self.embed_text(text))
    }

    fn embed_texts(&self, texts: Vec<String>) -> BoxFuture<Result<Vec<Embedding>, EmbeddingError>> {
        Box::pin(self.embed_texts(texts.into_iter().collect::<Vec<_>>()))
    }
}

/// Trait for embedding models that can generate embeddings for images.
pub trait ImageEmbeddingModel: Clone + Sync + Send {
    /// The maximum number of images that can be embedded in a single request.
    const MAX_DOCUMENTS: usize;

    /// The number of dimensions in the embedding vector.
    fn ndims(&self) -> usize;

    /// Embed multiple images in a single request from bytes.
    fn embed_images(
        &self,
        images: impl IntoIterator<Item = Vec<u8>> + Send,
    ) -> impl std::future::Future<Output = Result<Vec<Embedding>, EmbeddingError>> + Send;

    /// Embed a single image from bytes.
    fn embed_image<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> impl std::future::Future<Output = Result<Embedding, EmbeddingError>> + Send {
        async move {
            Ok(self
                .embed_images(vec![bytes.to_owned()])
                .await?
                .pop()
                .expect("There should be at least one embedding"))
        }
    }
}

/// Struct that holds a single document and its embedding.
#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Embedding {
    /// The document that was embedded. Used for debugging.
    pub document: String,
    /// The embedding vector
    pub vec: Vec<f64>,
}

impl PartialEq for Embedding {
    fn eq(&self, other: &Self) -> bool {
        self.document == other.document
    }
}

impl Eq for Embedding {}
