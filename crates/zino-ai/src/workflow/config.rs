use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_ms: 30000,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// 重试策略
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    /// 固定延迟重试
    FixedDelay { delay_ms: u64, max_retries: u32 },
    /// 指数退避重试
    ExponentialBackoff {
        initial_delay_ms: u64,
        max_delay_ms: u64,
        max_retries: u32,
    },
}

/// 缓存策略
#[derive(Debug, Clone)]
pub enum CachePolicy {
    /// 基于输入哈希缓存
    InputHash { ttl_seconds: u64 },
    /// 基于时间缓存
    TimeBased { ttl_seconds: u64 },
}

/// 节点支持的参数类型
#[derive(Debug, Clone, PartialEq)]
pub struct NodeParamTypes {
    pub needs_config: bool,
    pub needs_writer: bool,
    pub needs_store: bool,
    pub needs_runtime: bool,
}

impl Default for NodeParamTypes {
    fn default() -> Self {
        Self {
            needs_config: false,
            needs_writer: false,
            needs_store: false,
            needs_runtime: false,
        }
    }
}
