use std::sync::Arc;
use std::future::Future;
use async_trait::async_trait;

use super::{
    error::{WorkflowResult, WorkflowError},
    state::StateValue,
    config::{NodeConfig, NodeParamTypes},
    traits::{StateNode, NodeContext, ChannelWriter, Runtime},
};

/// 简单函数节点包装器（只有 state 参数）
pub struct FunctionNodeWrapper<F> {
    name: String,
    func: F,
    param_types: NodeParamTypes,
}

impl<F> FunctionNodeWrapper<F> {
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
    async fn execute(&self, state: StateValue, _context: &NodeContext) -> WorkflowResult<StateValue> {
        (self.func)(state)
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_supported_params(&self) -> NodeParamTypes {
        self.param_types.clone()
    }
}

/// 异步函数节点包装器
pub struct AsyncFunctionNodeWrapper<F> {
    name: String,
    func: F,
    param_types: NodeParamTypes,
}

impl<F> AsyncFunctionNodeWrapper<F> {
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
    async fn execute(&self, state: StateValue, _context: &NodeContext) -> WorkflowResult<StateValue> {
        (self.func)(state).await
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_supported_params(&self) -> NodeParamTypes {
        self.param_types.clone()
    }
}

/// 带配置的函数节点包装器
pub struct ConfigFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> ConfigFunctionNodeWrapper<F> {
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

#[async_trait]
impl<F> StateNode for ConfigFunctionNodeWrapper<F>
where
    F: Fn(StateValue, &NodeConfig) -> WorkflowResult<StateValue> + Send + Sync,
{
    async fn execute(&self, state: StateValue, context: &NodeContext) -> WorkflowResult<StateValue> {
        let config = context.config.as_ref()
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

/// 带配置和写入器的异步函数节点包装器
pub struct ConfigWriterAsyncFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> ConfigWriterAsyncFunctionNodeWrapper<F> {
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
    async fn execute(&self, state: StateValue, context: &NodeContext) -> WorkflowResult<StateValue> {
        let config = context.config.as_ref()
            .ok_or_else(|| WorkflowError::ConfigError("Config not available".to_string()))?;
        let writer = context.writer.as_ref()
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

/// 带运行时的函数节点包装器
pub struct RuntimeFunctionNodeWrapper<F> {
    name: String,
    func: F,
}

impl<F> RuntimeFunctionNodeWrapper<F> {
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
    async fn execute(&self, state: StateValue, context: &NodeContext) -> WorkflowResult<StateValue> {
        let runtime = context.runtime.as_ref()
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

/// 分支函数包装器
pub struct BranchFunctionWrapper<F> {
    name: String,
    func: F,
}

impl<F> BranchFunctionWrapper<F> {
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

#[async_trait]
impl<F> crate::workflow::traits::BranchPath for BranchFunctionWrapper<F>
where
    F: Fn(StateValue) -> crate::workflow::error::WorkflowResult<crate::workflow::traits::BranchResult> + Send + Sync,
{
    async fn route(&self, state: StateValue) -> WorkflowResult<crate::workflow::traits::BranchResult> {
        (self.func)(state)
    }
}
