use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::{
    config::{NodeConfig, NodeParamTypes},
    error::WorkflowResult,
    state::StateValue,
};

/// channel writer trait
#[async_trait]
pub trait ChannelWriter: Send + Sync {
    /// write a value to a channel
    async fn write(&self, channel_name: &str, value: StateValue) -> WorkflowResult<()>;
    /// write multiple values to channels
    async fn write_multiple(&self, writes: HashMap<String, StateValue>) -> WorkflowResult<()>;
}

/// An interface for node store.
#[async_trait]
pub trait NodeStore: Send + Sync {
    /// get a value from the store
    async fn get(&self, key: &str) -> WorkflowResult<Option<StateValue>>;
    /// set a value in the store
    async fn set(&self, key: String, value: StateValue) -> WorkflowResult<()>;
    /// delete a value from the store
    async fn delete(&self, key: &str) -> WorkflowResult<()>;
}

/// context runtime trait
#[async_trait]
pub trait Runtime: Send + Sync {
    /// get the current context
    async fn get_context(&self) -> WorkflowResult<HashMap<String, StateValue>>;
    /// set the current context
    async fn set_context(&self, context: HashMap<String, StateValue>) -> WorkflowResult<()>;

    /// get the current config
    async fn get_config(&self) -> WorkflowResult<NodeConfig>;
}

/// state node trait
#[async_trait]
pub trait StateNode: Send + Sync {
    /// execute the node with dynamic parameter injection
    async fn execute(&self, state: StateValue, context: &NodeContext)
    -> WorkflowResult<StateValue>;

    /// get the node name
    fn get_name(&self) -> &str;

    /// get the supported parameter types (for type checking)
    fn get_supported_params(&self) -> NodeParamTypes;
}

/// branch path trait
#[async_trait]
pub trait BranchPath: Send + Sync {
    /// route the state to the next branch
    async fn route(&self, state: StateValue) -> WorkflowResult<BranchResult>;
}

/// branch result
#[derive(Debug, Clone)]
pub enum BranchResult {
    /// single target
    Single(String),
    /// multiple targets
    Multiple(Vec<String>),
    /// send to a specific node
    Send(String, StateValue),
}

/// node execution context
#[derive(Clone)]
pub struct NodeContext {
    /// node configuration
    pub config: Option<NodeConfig>,
    /// channel writer
    pub writer: Option<Arc<dyn ChannelWriter>>,
    /// node store
    pub store: Option<Arc<dyn NodeStore>>,
    /// runtime
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
    /// create a new NodeContext
    pub fn new() -> Self {
        Self {
            config: None,
            writer: None,
            store: None,
            runtime: None,
        }
    }
    /// builder pattern methods
    pub fn with_config(mut self, config: NodeConfig) -> Self {
        self.config = Some(config);
        self
    }
    /// with_writer sets the channel writer
    pub fn with_writer(mut self, writer: Arc<dyn ChannelWriter>) -> Self {
        self.writer = Some(writer);
        self
    }
    /// with_store sets the node store
    pub fn with_store(mut self, store: Arc<dyn NodeStore>) -> Self {
        self.store = Some(store);
        self
    }
    /// with_runtime sets the runtime
    pub fn with_runtime(mut self, runtime: Arc<dyn Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }
}
impl Default for NodeContext {
    fn default() -> Self {
        Self::new()
    }
}
