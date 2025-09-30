//! Universal embedding module.
//!
//! This module provides universal embedding functionality for converting text
//! and messages into vector representations across different AI service providers.

pub mod embedding;
pub mod error;
pub mod rag;
pub mod vector_store;

pub use embedding::{Embedding, EmbeddingError, EmbeddingModel};
pub use error::{VectorError, VectorResult};
pub use rag::{KnowledgeChunk, RAGSystem, SearchResult, VectorStore};
pub use vector_store::{FileVectorStore, InMemoryVectorStore, VectorStoreFactory};

/// Re-export commonly used types
pub use crate::memory::Message;
