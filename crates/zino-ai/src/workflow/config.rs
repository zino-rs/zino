use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Maximum retry attempts.
    pub max_retries: u32,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Tags associated with the node.
    pub tags: Vec<String>,
    /// Metadata for the node.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NodeConfig {
    /// Creates a new NodeConfig.
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

/// Retry policy.
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    /// Fixed delay retry.
    FixedDelay {
        /// Delay in milliseconds.
        delay_ms: u64,
        /// Maximum retry attempts.
        max_retries: u32,
    },
    /// Exponential backoff retry.
    ExponentialBackoff {
        /// Initial delay in milliseconds.
        initial_delay_ms: u64,
        /// Maximum delay in milliseconds.
        max_delay_ms: u64,
        /// Maximum retry attempts.
        max_retries: u32,
    },
}

/// Cache policy.
#[derive(Debug, Clone)]
pub enum CachePolicy {
    /// Input hash based cache.
    InputHash {
        /// Time to live in seconds.
        ttl_seconds: u64,
    },
    /// Time based cache.
    TimeBased {
        /// Time to live in seconds.
        ttl_seconds: u64,
    },
}

/// Supported node parameter types.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeParamTypes {
    /// Whether the node needs configuration.
    pub needs_config: bool,
    /// Whether the node needs a writer.
    pub needs_writer: bool,
    /// Whether the node needs a store.
    pub needs_store: bool,
    /// Whether the node needs a runtime.
    pub needs_runtime: bool,
}

impl NodeParamTypes {
    /// Creates a new NodeParamTypes.
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
