//! RAG (Retrieval-Augmented Generation) functionality.
//!
//! This module provides RAG capabilities by combining embedding vectors
//! with memory system for enhanced AI responses.

use crate::embeddings::embedding::EmbeddingModel;
use crate::embeddings::error::{VectorError, VectorResult};
use crate::embeddings::vector_store::VectorStoreFactory;
use crate::memory::{MemoryManager, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RAG system for enhanced AI responses.
///
/// This struct combines embedding vectors with memory system to provide
/// retrieval-augmented generation capabilities.
#[derive(Debug)]
pub struct RAGSystem {
    /// Universal embedding model for vector generation
    embedding_model: EmbeddingModel,
    /// Memory manager for conversation history
    memory: MemoryManager,
    /// Knowledge base (document chunks with embeddings)
    knowledge_base: Vec<KnowledgeChunk>,
    /// Similarity threshold for retrieval
    similarity_threshold: f32,
    /// Vector store for efficient similarity search
    vector_store: Option<Box<dyn VectorStore>>,
}

/// Trait for vector storage operations
pub trait VectorStore: Send + Sync + std::fmt::Debug {
    /// Insert a vector with metadata
    fn insert(
        &self,
        id: &str,
        vector: Vec<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Search for similar vectors
    fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;

    /// Delete a vector by ID
    fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Get vector by ID
    fn get(&self, id: &str) -> Result<Option<Vec<f32>>, Box<dyn std::error::Error>>;
}

/// A chunk of knowledge with its embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    /// Unique identifier
    pub id: String,
    /// Text content
    pub content: String,
    /// Vector embedding
    pub embedding: Vec<f32>,
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}

impl RAGSystem {
    /// Create a new RAG system with embedding model.
    ///
    /// # Arguments
    /// * `embedding_model` - Universal embedding model
    /// * `similarity_threshold` - Minimum similarity for retrieval
    ///
    /// # Returns
    /// * `Self` - New RAG system instance
    pub fn new(embedding_model: EmbeddingModel, similarity_threshold: f32) -> Self {
        use crate::memory::BufferMemory;
        use std::sync::Arc;

        let memory = Arc::new(BufferMemory::new());

        Self {
            embedding_model,
            memory: MemoryManager::new(memory),
            knowledge_base: Vec::new(),
            similarity_threshold,
            vector_store: None,
        }
    }

    /// Create a new RAG system with in-memory vector store
    ///
    /// # Arguments
    /// * `embedding_model` - Universal embedding model
    /// * `similarity_threshold` - Minimum similarity threshold
    ///
    /// # Returns
    /// * `Self` - New RAG system instance
    pub fn with_in_memory_store(
        embedding_model: EmbeddingModel,
        similarity_threshold: f32,
    ) -> Self {
        let vector_store = VectorStoreFactory::create_in_memory();
        Self::with_vector_store(embedding_model, vector_store, similarity_threshold)
    }

    /// Create a new RAG system with file-based vector store
    ///
    /// # Arguments
    /// * `embedding_model` - Universal embedding model
    /// * `file_path` - Path to store vectors
    /// * `similarity_threshold` - Minimum similarity threshold
    ///
    /// # Returns
    /// * `Self` - New RAG system instance
    pub async fn with_file_store(
        embedding_model: EmbeddingModel,
        file_path: &str,
        similarity_threshold: f32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let vector_store = VectorStoreFactory::create_file_based(file_path)?;
        Ok(Self::with_vector_store(
            embedding_model,
            vector_store,
            similarity_threshold,
        ))
    }

    /// Create a new RAG system with vector store
    ///
    /// # Arguments
    /// * `embedding_model` - Universal embedding model
    /// * `vector_store` - Vector store for efficient search
    /// * `similarity_threshold` - Minimum similarity for retrieval
    ///
    /// # Returns
    /// * `Self` - New RAG system instance
    pub fn with_vector_store(
        embedding_model: EmbeddingModel,
        vector_store: Box<dyn VectorStore>,
        similarity_threshold: f32,
    ) -> Self {
        use crate::memory::BufferMemory;
        use std::sync::Arc;

        let memory = Arc::new(BufferMemory::new());

        Self {
            embedding_model,
            memory: MemoryManager::new(memory),
            knowledge_base: Vec::new(),
            similarity_threshold,
            vector_store: Some(vector_store),
        }
    }

    /// Add a document to the knowledge base.
    ///
    /// # Arguments
    /// * `content` - Document content
    /// * `metadata` - Optional metadata
    /// * `model_name` - Embedding model name to use
    ///
    /// # Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - ID of the added chunk
    pub async fn add_document(
        &mut self,
        content: String,
        metadata: Option<HashMap<String, String>>,
        model_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = format!("doc_{}", uuid::Uuid::new_v4());

        // Use the universal embedding model
        let request_body = serde_json::json!({
            "input": content,
            "model": model_name
        });
        let embedding_result = self.embedding_model.embed(request_body).await?;
        let embedding = self.extract_embedding_from_response(embedding_result)?;

        let chunk = KnowledgeChunk {
            id: id.clone(),
            content,
            embedding: embedding.clone(),
            metadata: metadata.clone(),
        };

        // Add to knowledge base
        self.knowledge_base.push(chunk);

        // Add to vector store if available
        if let Some(store) = &self.vector_store {
            store.insert(&id, embedding, metadata)?;
        }

        Ok(id)
    }

    /// Extract embedding vector from response
    fn extract_embedding_from_response(
        &self,
        response: crate::embeddings::embedding::Embedding<serde_json::Value>,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // This is a generic implementation - actual behavior depends on the provider
        let embedding = response.raw_response["data"][0]["embedding"]
            .as_array()
            .ok_or("Missing embedding data")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        Ok(embedding)
    }

    /// Retrieve relevant documents for a query.
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `limit` - Maximum number of results
    /// * `model_name` - Embedding model name to use
    ///
    /// # Returns
    /// * `Result<Vec<&KnowledgeChunk>, Box<dyn std::error::Error>>` - Relevant document chunks
    pub async fn retrieve(
        &self,
        query: &str,
        limit: usize,
        model_name: &str,
    ) -> Result<Vec<&KnowledgeChunk>, Box<dyn std::error::Error>> {
        // Use vector store if available, otherwise fall back to in-memory search
        if let Some(store) = &self.vector_store {
            self.retrieve_with_vector_store(query, limit, model_name, store.as_ref())
                .await
        } else {
            self.retrieve_in_memory(query, limit, model_name).await
        }
    }

    /// Retrieve using vector store
    async fn retrieve_with_vector_store(
        &self,
        query: &str,
        limit: usize,
        model_name: &str,
        store: &dyn VectorStore,
    ) -> Result<Vec<&KnowledgeChunk>, Box<dyn std::error::Error>> {
        // Get query embedding
        let request_body = serde_json::json!({
            "input": query,
            "model": model_name
        });
        let query_embedding_result = self.embedding_model.embed(request_body).await?;
        let query_embedding = self.extract_embedding_from_response(query_embedding_result)?;

        // Search in vector store
        let search_results = store.search(query_embedding, limit, self.similarity_threshold)?;

        // Convert to knowledge chunks
        let mut chunks = Vec::new();
        for result in search_results {
            if let Some(chunk) = self.knowledge_base.iter().find(|c| c.id == result.id) {
                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Retrieve using in-memory search
    async fn retrieve_in_memory(
        &self,
        query: &str,
        limit: usize,
        model_name: &str,
    ) -> Result<Vec<&KnowledgeChunk>, Box<dyn std::error::Error>> {
        // Get query embedding
        let request_body = serde_json::json!({
            "input": query,
            "model": model_name
        });
        let query_embedding_result = self.embedding_model.embed(request_body).await?;
        let query_embedding = self.extract_embedding_from_response(query_embedding_result)?;

        let mut results = Vec::new();

        for chunk in &self.knowledge_base {
            let similarity = self.cosine_similarity(&query_embedding, &chunk.embedding);
            if similarity >= self.similarity_threshold {
                results.push((chunk, similarity));
            }
        }

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top results
        Ok(results
            .into_iter()
            .take(limit)
            .map(|(chunk, _)| chunk)
            .collect())
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Add a conversation message to memory.
    ///
    /// # Arguments
    /// * `message` - The message to add
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error
    pub fn add_message(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.memory.add_message(message)?;
        Ok(())
    }

    /// Generate a response using RAG.
    ///
    /// # Arguments
    /// * `query` - User query
    /// * `context_limit` - Maximum number of context chunks
    /// * `model_name` - Embedding model name to use
    ///
    /// # Returns
    /// * `VectorResult<String>` - Generated response
    pub async fn generate_response(
        &self,
        query: &str,
        context_limit: usize,
        model_name: &str,
    ) -> VectorResult<String> {
        // Retrieve relevant context
        let relevant_chunks = self
            .retrieve(query, context_limit, model_name)
            .await
            .map_err(|e| VectorError::IoError(e.to_string()))?;

        // Build context from retrieved chunks
        let mut context = String::new();
        for (i, chunk) in relevant_chunks.iter().enumerate() {
            context.push_str(&format!("Context {}: {}\n", i + 1, chunk.content));
        }

        // Get conversation history
        let history = self
            .memory
            .get_formatted_history()
            .unwrap_or_else(|_| "No conversation history".to_string());

        // Generate response (simplified - in real implementation, this would call an LLM)
        let response = self.generate_simple_response(query, &context, &history);

        Ok(response)
    }

    /// Generate a simple response based on context.
    ///
    /// # Arguments
    /// * `query` - User query
    /// * `context` - Retrieved context
    /// * `_history` - Conversation history (unused for now)
    ///
    /// # Returns
    /// * `String` - Generated response
    fn generate_simple_response(&self, query: &str, context: &str, _history: &str) -> String {
        // This is a simplified response generator
        // In a real implementation, this would call an LLM API

        if context.is_empty() {
            format!(
                "I don't have specific information about \"{}\". Could you provide more details?",
                query
            )
        } else {
            format!(
                "Based on the available information:\n\n{}\n\nRegarding your question \"{}\", here's what I found: The retrieved context provides relevant information that can help answer your query.",
                context.trim(),
                query
            )
        }
    }
}

/// Search result from RAG system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document chunk ID
    pub id: String,
    /// Content text
    pub content: String,
    /// Similarity score
    pub score: f32,
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}
