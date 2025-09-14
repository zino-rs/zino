use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use super::{
    error::WorkflowResult,
    state::StateValue,
    config::{NodeConfig, NodeParamTypes},
};

/// 通道写入器 trait
#[async_trait]
pub trait ChannelWriter: Send + Sync {
    async fn write(&self, channel_name: &str, value: StateValue) -> WorkflowResult<()>;
    async fn write_multiple(&self, writes: HashMap<String, StateValue>) -> WorkflowResult<()>;
}

/// 节点存储 trait
#[async_trait]
pub trait NodeStore: Send + Sync {
    async fn get(&self, key: &str) -> WorkflowResult<Option<StateValue>>;
    async fn set(&self, key: String, value: StateValue) -> WorkflowResult<()>;
    async fn delete(&self, key: &str) -> WorkflowResult<()>;
}

/// 运行时上下文 trait
#[async_trait]
pub trait Runtime: Send + Sync {
    async fn get_context(&self) -> WorkflowResult<HashMap<String, StateValue>>;
    async fn set_context(&self, context: HashMap<String, StateValue>) -> WorkflowResult<()>;
    async fn get_config(&self) -> WorkflowResult<NodeConfig>;
}

/// 统一的节点执行器 trait - 对应 Python 的 StateNode
#[async_trait]
pub trait StateNode: Send + Sync {
    /// 执行节点，支持动态参数注入
    async fn execute(&self, state: StateValue, context: &NodeContext) -> WorkflowResult<StateValue>;
    
    /// 获取节点名称
    fn get_name(&self) -> &str;
    
    /// 获取支持的参数类型（用于类型检查）
    fn get_supported_params(&self) -> NodeParamTypes;
}

/// 分支路径 trait
#[async_trait]
pub trait BranchPath: Send + Sync {
    async fn route(&self, state: StateValue) -> WorkflowResult<BranchResult>;
}

/// 分支结果
#[derive(Debug, Clone)]
pub enum BranchResult {
    /// 单个目标
    Single(String),
    /// 多个目标
    Multiple(Vec<String>),
    /// 发送到特定节点
    Send(String, StateValue),
}

/// 节点执行上下文
#[derive(Clone)]
pub struct NodeContext {
    pub config: Option<NodeConfig>,
    pub writer: Option<Arc<dyn ChannelWriter>>,
    pub store: Option<Arc<dyn NodeStore>>,
    pub runtime: Option<Arc<dyn Runtime>>,
}

impl std::fmt::Debug for NodeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeContext")
            .field("config", &self.config)
            .field("writer", &"<dyn ChannelWriter>")
            .field("store", &"<dyn NodeStore>")
            .field("runtime", &"<dyn Runtime>")
            .finish()
    }
}

impl NodeContext {
    pub fn new() -> Self {
        Self {
            config: None,
            writer: None,
            store: None,
            runtime: None,
        }
    }
    
    pub fn with_config(mut self, config: NodeConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn with_writer(mut self, writer: Arc<dyn ChannelWriter>) -> Self {
        self.writer = Some(writer);
        self
    }
    
    pub fn with_store(mut self, store: Arc<dyn NodeStore>) -> Self {
        self.store = Some(store);
        self
    }
    
    pub fn with_runtime(mut self, runtime: Arc<dyn Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }
}
