use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::{
    config::{NodeConfig, NodeParamTypes},
    error::WorkflowResult,
    state::StateValue,
};

/// Trait for writing values to workflow channels.
///
/// `ChannelWriter` provides an abstraction for writing state values to channels,
/// supporting both single and batch operations.
#[async_trait]
pub trait ChannelWriter: Send + Sync {
    /// Writes a value to a specific channel.
    async fn write(&self, channel_name: &str, value: StateValue) -> WorkflowResult<()>;

    /// Writes multiple values to different channels in a single operation.
    async fn write_multiple(&self, writes: HashMap<String, StateValue>) -> WorkflowResult<()>;
}

/// Trait for persistent storage of workflow state values.
///
/// `NodeStore` provides an abstraction for storing and retrieving state values
/// that need to persist across workflow executions or be shared between nodes.
#[async_trait]
pub trait NodeStore: Send + Sync {
    /// Retrieves a value from the store by key.
    async fn get(&self, key: &str) -> WorkflowResult<Option<StateValue>>;

    /// Stores a value in the store with the given key.
    async fn set(&self, key: String, value: StateValue) -> WorkflowResult<()>;

    /// Deletes a value from the store by key.
    async fn delete(&self, key: &str) -> WorkflowResult<()>;
}

/// Trait for runtime context management in workflows.
///
/// `Runtime` provides access to execution context and configuration
/// that can be shared across workflow nodes.
#[async_trait]
pub trait Runtime: Send + Sync {
    /// Gets the current execution context.
    async fn get_context(&self) -> WorkflowResult<HashMap<String, StateValue>>;

    /// Sets the current execution context.
    async fn set_context(&self, context: HashMap<String, StateValue>) -> WorkflowResult<()>;

    /// Gets the current node configuration.
    async fn get_config(&self) -> WorkflowResult<NodeConfig>;
}

/// Trait for implementing workflow nodes.
///
/// `StateNode` represents a single processing unit in a workflow that
/// transforms input state into output state.
#[async_trait]
pub trait StateNode: Send + Sync {
    /// Executes the node with the given input state and context.
    async fn execute(&self, state: StateValue, context: &NodeContext)
    -> WorkflowResult<StateValue>;

    /// Gets the name of this node.
    fn get_name(&self) -> &str;

    /// Gets the parameter types supported by this node.
    ///
    /// This is used for type checking and validation during workflow construction.
    fn get_supported_params(&self) -> NodeParamTypes;
}

/// Trait for implementing conditional routing in workflows.
///
/// `BranchPath` represents a decision point in a workflow that determines
/// which node(s) to execute next based on the current state.
#[async_trait]
pub trait BranchPath: Send + Sync {
    /// Routes the state to the next branch(es) in the workflow.
    async fn route(&self, state: StateValue) -> WorkflowResult<BranchResult>;
}

/// Represents the result of a branch routing decision.
///
/// `BranchResult` indicates which node(s) should be executed next
/// and optionally provides modified state for the next execution.
#[derive(Debug, Clone)]
pub enum BranchResult {
    /// Route to a single target node.
    Single(String),
    /// Route to multiple target nodes.
    Multiple(Vec<String>),
    /// Route to a specific node with modified state.
    Send(String, StateValue),
}

/// Execution context for workflow nodes.
///
/// `NodeContext` provides access to configuration, services, and runtime
/// information that nodes need during execution.
#[derive(Clone)]
pub struct NodeContext {
    /// Node configuration settings.
    pub config: Option<NodeConfig>,
    /// Channel writer for inter-node communication.
    pub writer: Option<Arc<dyn ChannelWriter>>,
    /// Persistent storage for state values.
    pub store: Option<Arc<dyn NodeStore>>,
    /// Runtime context and configuration access.
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
    /// Creates a new empty NodeContext.
    pub fn new() -> Self {
        Self {
            config: None,
            writer: None,
            store: None,
            runtime: None,
        }
    }

    /// Sets the node configuration.
    pub fn with_config(mut self, config: NodeConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the channel writer.
    pub fn with_writer(mut self, writer: Arc<dyn ChannelWriter>) -> Self {
        self.writer = Some(writer);
        self
    }

    /// Sets the node store.
    pub fn with_store(mut self, store: Arc<dyn NodeStore>) -> Self {
        self.store = Some(store);
        self
    }

    /// Sets the runtime.
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
