use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

use super::{
    config::{NodeConfig, NodeParamTypes},
    error::{WorkflowError, WorkflowResult},
    state::StateValue,
    traits::{ChannelWriter, NodeContext, Runtime, StateNode},
};

/// Wrapper for simple function-based workflow nodes.
///
/// `FunctionNodeWrapper` adapts a simple function into a workflow node,
/// providing a convenient way to create nodes from existing functions.
pub struct FunctionNodeWrapper<F> {
    name: String,
    func: F,
    param_types: NodeParamTypes,
}

impl<F> FunctionNodeWrapper<F> {
    /// Creates a new FunctionNodeWrapper with the given name and function.
    pub fn new(name: String, func: F) -> Self {
        Self {
            name,
            func,
            param_types: NodeParamTypes::default(),
        }
    }
}

#[async_trait]
impl<F> StateNode for FunctionNodeWrapper<F>
where
    F: Fn(StateValue) -> WorkflowResult<StateValue> + Send + Sync,
{
    async fn execute(
        &self,
        state: StateValue,
        _context: &NodeContext,
    ) -> WorkflowResult<StateValue> {
        (self.func)(state)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_supported_params(&self) -> NodeParamTypes {
        self.param_types.clone()
    }
}

/// Wrapper for async function-based workflow nodes.
///
/// `AsyncFunctionNodeWrapper` adapts an async function into a workflow node,
/// enabling asynchronous processing within workflows.
pub struct AsyncFunctionNodeWrapper<F> {
    name: String,
    func: F,
    param_types: NodeParamTypes,
}

impl<F> AsyncFunctionNodeWrapper<F> {
    /// Creates a new AsyncFunctionNodeWrapper with the given name and async function.
    pub fn new(name: String, func: F) -> Self {
        Self {
            name,
            func,
            param_types: NodeParamTypes::default(),
        }
    }
}

#[async_trait]
impl<F, Fut> StateNode for AsyncFunctionNodeWrapper<F>
where
    F: Fn(StateValue) -> Fut + Send + Sync,
    Fut: Future<Output = WorkflowResult<StateValue>> + Send,
{
    async fn execute(
        &self,
        state: StateValue,
        _context: &NodeContext,
    ) -> WorkflowResult<StateValue> {
        (self.func)(state).await
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_supported_params(&self) -> NodeParamTypes {
        self.param_types.clone()
    }
}

/// Wrapper for configuration-aware workflow nodes.
///
/// `ConfigFunctionNodeWrapper` adapts a function that needs configuration
/// into a workflow node, providing access to node configuration during execution.
pub struct ConfigFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> ConfigFunctionNodeWrapper<F> {
    /// Creates a new ConfigFunctionNodeWrapper with the given name and function.
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

#[async_trait]
impl<F> StateNode for ConfigFunctionNodeWrapper<F>
where
    F: Fn(StateValue, &NodeConfig) -> WorkflowResult<StateValue> + Send + Sync,
{
    async fn execute(
        &self,
        state: StateValue,
        context: &NodeContext,
    ) -> WorkflowResult<StateValue> {
        let config = context
            .config
            .as_ref()
            .ok_or_else(|| WorkflowError::ConfigError("Config not available".to_string()))?;
        (self.func)(state, config)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_supported_params(&self) -> NodeParamTypes {
        NodeParamTypes {
            needs_config: true,
            needs_writer: false,
            needs_store: false,
            needs_runtime: false,
        }
    }
}

/// Wrapper for async configuration and writer-aware workflow nodes.
///
/// `ConfigWriterAsyncFunctionNodeWrapper` adapts an async function that needs
/// both configuration and channel writer access into a workflow node.
pub struct ConfigWriterAsyncFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> ConfigWriterAsyncFunctionNodeWrapper<F> {
    /// Creates a new ConfigWriterAsyncFunctionNodeWrapper with the given name and function.
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

#[async_trait]
impl<F, Fut> StateNode for ConfigWriterAsyncFunctionNodeWrapper<F>
where
    F: Fn(StateValue, &NodeConfig, Arc<dyn ChannelWriter>) -> Fut + Send + Sync,
    Fut: Future<Output = WorkflowResult<StateValue>> + Send,
{
    async fn execute(
        &self,
        state: StateValue,
        context: &NodeContext,
    ) -> WorkflowResult<StateValue> {
        let config = context
            .config
            .as_ref()
            .ok_or_else(|| WorkflowError::ConfigError("Config not available".to_string()))?;
        let writer = context
            .writer
            .as_ref()
            .ok_or_else(|| WorkflowError::ConfigError("Writer not available".to_string()))?
            .clone();
        (self.func)(state, config, writer).await
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_supported_params(&self) -> NodeParamTypes {
        NodeParamTypes {
            needs_config: true,
            needs_writer: true,
            needs_store: false,
            needs_runtime: false,
        }
    }
}

/// Wrapper for runtime-aware workflow nodes.
///
/// `RuntimeFunctionNodeWrapper` adapts an async function that needs
/// runtime context access into a workflow node.
pub struct RuntimeFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> RuntimeFunctionNodeWrapper<F> {
    /// Creates a new RuntimeFunctionNodeWrapper with the given name and function.
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

#[async_trait]
impl<F, Fut> StateNode for RuntimeFunctionNodeWrapper<F>
where
    F: Fn(StateValue, Arc<dyn Runtime>) -> Fut + Send + Sync,
    Fut: Future<Output = WorkflowResult<StateValue>> + Send,
{
    async fn execute(
        &self,
        state: StateValue,
        context: &NodeContext,
    ) -> WorkflowResult<StateValue> {
        let runtime = context
            .runtime
            .as_ref()
            .ok_or_else(|| WorkflowError::ConfigError("Runtime not available".to_string()))?
            .clone();
        (self.func)(state, runtime).await
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_supported_params(&self) -> NodeParamTypes {
        NodeParamTypes {
            needs_config: false,
            needs_writer: false,
            needs_store: false,
            needs_runtime: true,
        }
    }
}

/// Wrapper for branch function-based workflow nodes.
///
/// `BranchFunctionWrapper` adapts a function that performs conditional routing
/// into a workflow node, enabling decision-making within workflows.
pub struct BranchFunctionWrapper<F> {
    #[allow(dead_code)]
    name: String,
    func: F,
}

impl<F> BranchFunctionWrapper<F> {
    /// Creates a new BranchFunctionWrapper with the given name and function.
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

#[async_trait]
impl<F> crate::workflow::traits::BranchPath for BranchFunctionWrapper<F>
where
    F: Fn(
            StateValue,
        ) -> crate::workflow::error::WorkflowResult<crate::workflow::traits::BranchResult>
        + Send
        + Sync,
{
    async fn route(
        &self,
        state: StateValue,
    ) -> WorkflowResult<crate::workflow::traits::BranchResult> {
        (self.func)(state)
    }
}
