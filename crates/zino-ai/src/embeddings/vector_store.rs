//! Simple vector storage implementations for RAG.
//!
//! This module provides lightweight vector storage options that don't require

use super::rag::{SearchResult, VectorStore};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// Type alias for vector data stored in memory.
///
/// This represents a tuple of (vector, metadata) where:
/// - vector: The embedding vector as Vec<f32>
/// - metadata: Optional metadata as key-value pairs
type VectorData = (Vec<f32>, Option<HashMap<String, String>>);

/// Type alias for the internal storage structure.
///
/// This maps document IDs to their vector data.
type VectorStorage = Arc<Mutex<HashMap<String, VectorData>>>;

/// Simple in-memory vector store
#[derive(Debug, Clone)]
pub struct InMemoryVectorStore {
    vectors: VectorStorage,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryVectorStore {
    /// Creates a new empty in-memory vector store.
    ///
    /// This is equivalent to calling `InMemoryVectorStore::new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore for InMemoryVectorStore {
    fn insert(
        &self,
        id: &str,
        vector: Vec<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut vectors = self.vectors.lock().unwrap();
        vectors.insert(id.to_string(), (vector, metadata));
        Ok(())
    }

    fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let vectors = self.vectors.lock().unwrap();
        let mut results = Vec::new();

        for (id, (vector, metadata)) in vectors.iter() {
            let similarity = cosine_similarity(&query_vector, vector);
            if similarity >= threshold {
                results.push(SearchResult {
                    id: id.clone(),
                    content: metadata
                        .as_ref()
                        .and_then(|m| m.get("content").cloned())
                        .unwrap_or_default(),
                    score: similarity,
                    metadata: metadata.clone(),
                });
            }
        }

        // Sort by similarity score (descending) and limit results
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    fn delete(&self, id: &str) -> Result<(), Box<dyn Error>> {
        let mut vectors = self.vectors.lock().unwrap();
        vectors.remove(id);
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<Vec<f32>>, Box<dyn Error>> {
        let vectors = self.vectors.lock().unwrap();
        Ok(vectors.get(id).map(|(vector, _)| vector.clone()))
    }
}

/// File-based vector store using JSON serialization
#[derive(Debug)]
pub struct FileVectorStore {
    file_path: String,
    vectors: VectorStorage,
}

impl FileVectorStore {
    /// Create a new file-based vector store
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut vectors = HashMap::new();

        // Try to load existing data
        if std::path::Path::new(file_path).exists() {
            let data = std::fs::read_to_string(file_path)?;
            let loaded: HashMap<String, VectorData> =
                serde_json::from_str(&data).unwrap_or_default();
            vectors = loaded;
        }

        Ok(Self {
            file_path: file_path.to_string(),
            vectors: Arc::new(Mutex::new(vectors)),
        })
    }

    /// Save vectors to file
    fn save(&self) -> Result<(), Box<dyn Error>> {
        let vectors = self.vectors.lock().unwrap();
        let data = serde_json::to_string_pretty(&*vectors)?;
        std::fs::write(&self.file_path, data)?;
        Ok(())
    }
}

impl VectorStore for FileVectorStore {
    fn insert(
        &self,
        id: &str,
        vector: Vec<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut vectors = self.vectors.lock().unwrap();
        vectors.insert(id.to_string(), (vector, metadata));
        self.save()?;
        Ok(())
    }

    fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let vectors = self.vectors.lock().unwrap();
        let mut results = Vec::new();

        for (id, (vector, metadata)) in vectors.iter() {
            let similarity = cosine_similarity(&query_vector, vector);
            if similarity >= threshold {
                results.push(SearchResult {
                    id: id.clone(),
                    content: metadata
                        .as_ref()
                        .and_then(|m| m.get("content").cloned())
                        .unwrap_or_default(),
                    score: similarity,
                    metadata: metadata.clone(),
                });
            }
        }

        // Sort by similarity score (descending) and limit results
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    fn delete(&self, id: &str) -> Result<(), Box<dyn Error>> {
        let mut vectors = self.vectors.lock().unwrap();
        vectors.remove(id);
        self.save()?;
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<Vec<f32>>, Box<dyn Error>> {
        let vectors = self.vectors.lock().unwrap();
        Ok(vectors.get(id).map(|(vector, _)| vector.clone()))
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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

/// Vector store factory for creating different types of stores
pub struct VectorStoreFactory;

impl VectorStoreFactory {
    /// Create an in-memory vector store
    pub fn create_in_memory() -> Box<dyn VectorStore> {
        Box::new(InMemoryVectorStore::new())
    }

    /// Create a file-based vector store
    pub fn create_file_based(file_path: &str) -> Result<Box<dyn VectorStore>, Box<dyn Error>> {
        Ok(Box::new(FileVectorStore::new(file_path)?))
    }
}
