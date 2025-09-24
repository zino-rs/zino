use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

use super::{
    config::{NodeConfig, NodeParamTypes},
    error::{WorkflowError, WorkflowResult},
    state::StateValue,
    traits::{ChannelWriter, NodeContext, Runtime, StateNode},
};

/// simple function node wrapper
pub struct FunctionNodeWrapper<F> {
    name: String,
    func: F,
    param_types: NodeParamTypes,
}

impl<F> FunctionNodeWrapper<F> {
    /// create a new FunctionNodeWrapper
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

/// Async function node wrapper
pub struct AsyncFunctionNodeWrapper<F> {
    name: String,
    func: F,
    param_types: NodeParamTypes,
}

impl<F> AsyncFunctionNodeWrapper<F> {
    /// create a new AsyncFunctionNodeWrapper
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

/// Config function node wrapper
pub struct ConfigFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> ConfigFunctionNodeWrapper<F> {
    /// create a new ConfigFunctionNodeWrapper
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

/// A config and writer async function node wrapper
pub struct ConfigWriterAsyncFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> ConfigWriterAsyncFunctionNodeWrapper<F> {
    /// create a new ConfigWriterAsyncFunctionNodeWrapper
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

/// A runtime function node wrapper
pub struct RuntimeFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> RuntimeFunctionNodeWrapper<F> {
    /// create a new RuntimeFunctionNodeWrapper
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

/// branch wrapper
pub struct BranchFunctionWrapper<F> {
    #[allow(dead_code)]
    name: String,
    func: F,
}

impl<F> BranchFunctionWrapper<F> {
    /// create a new BranchFunctionWrapper
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
