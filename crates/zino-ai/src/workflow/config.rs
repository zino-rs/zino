use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration settings for workflow nodes.
///
/// `NodeConfig` provides configuration options for individual nodes,
/// including retry policies, timeouts, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Maximum number of retry attempts for failed operations.
    pub max_retries: u32,
    /// Timeout duration in milliseconds.
    pub timeout_ms: u64,
    /// Tags associated with the node for categorization.
    pub tags: Vec<String>,
    /// Additional metadata for the node.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NodeConfig {
    /// Creates a new NodeConfig with the given parameters.
    pub fn new(
        max_retries: u32,
        timeout_ms: u64,
        tags: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            max_retries,
            timeout_ms,
            tags,
            metadata,
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self::new(3, 30000, Vec::new(), HashMap::new())
    }
}

/// Retry policy for handling failed operations.
///
/// `RetryPolicy` defines how failed operations should be retried,
/// supporting both fixed delay and exponential backoff strategies.
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    /// Fixed delay retry with constant delay between attempts.
    FixedDelay {
        /// Delay in milliseconds between retry attempts.
        delay_ms: u64,
        /// Maximum number of retry attempts.
        max_retries: u32,
    },
    /// Exponential backoff retry with increasing delay between attempts.
    ExponentialBackoff {
        /// Initial delay in milliseconds for the first retry.
        initial_delay_ms: u64,
        /// Maximum delay in milliseconds (caps the exponential growth).
        max_delay_ms: u64,
        /// Maximum number of retry attempts.
        max_retries: u32,
    },
}

/// Cache policy for storing and retrieving cached results.
///
/// `CachePolicy` defines how cached results should be stored and retrieved,
/// supporting both input hash-based and time-based caching strategies.
#[derive(Debug, Clone)]
pub enum CachePolicy {
    /// Input hash based cache that uses input hash as the cache key.
    InputHash {
        /// Time to live in seconds for cached results.
        ttl_seconds: u64,
    },
    /// Time based cache that uses time-based expiration.
    TimeBased {
        /// Time to live in seconds for cached results.
        ttl_seconds: u64,
    },
}

/// Parameter types supported by workflow nodes.
///
/// `NodeParamTypes` indicates what types of parameters a node requires
/// during execution, enabling type checking and validation.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeParamTypes {
    /// Whether the node needs configuration parameters.
    pub needs_config: bool,
    /// Whether the node needs a channel writer.
    pub needs_writer: bool,
    /// Whether the node needs a persistent store.
    pub needs_store: bool,
    /// Whether the node needs runtime context.
    pub needs_runtime: bool,
}

impl NodeParamTypes {
    /// Creates a new NodeParamTypes with the specified parameter requirements.
    pub fn new(
        needs_config: bool,
        needs_writer: bool,
        needs_store: bool,
        needs_runtime: bool,
    ) -> Self {
        Self {
            needs_config,
            needs_writer,
            needs_store,
            needs_runtime,
        }
    }
}

impl Default for NodeParamTypes {
    fn default() -> Self {
        Self::new(false, false, false, false)
    }
}
