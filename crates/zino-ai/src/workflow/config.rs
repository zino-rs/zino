use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// node settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// maximum retry attempts
    pub max_retries: u32,
    /// timeout in milliseconds
    pub timeout_ms: u64,
    /// tags associated with the node
    pub tags: Vec<String>,
    /// metadata for the node
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NodeConfig {
    /// create a new NodeConfig
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

/// retry policy
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    /// fixed delay retry
    FixedDelay {
        /// delay in milliseconds
        delay_ms: u64,
        /// maximum retry attempts
        max_retries: u32,
    },
    /// exponential backoff retry
    ExponentialBackoff {
        /// initial delay in milliseconds
        initial_delay_ms: u64,
        /// maximum delay in milliseconds
        max_delay_ms: u64,
        /// maximum retry attempts
        max_retries: u32,
    },
}

/// cache policy
#[derive(Debug, Clone)]
pub enum CachePolicy {
    /// input hash based cache
    InputHash {
        /// time to live in seconds
        ttl_seconds: u64,
    },
    /// time based cache
    TimeBased {
        /// time to live in seconds
        ttl_seconds: u64,
    },
}

/// node supported parameter types
#[derive(Debug, Clone, PartialEq)]
pub struct NodeParamTypes {
    /// whether the node needs configuration
    pub needs_config: bool,
    /// whether the node needs a writer
    pub needs_writer: bool,
    /// whether the node needs a store
    pub needs_store: bool,
    /// whether the node needs a runtime
    pub needs_runtime: bool,
}

impl NodeParamTypes {
    /// create a new NodeParamTypes
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
