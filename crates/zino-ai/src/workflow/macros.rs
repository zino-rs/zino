#[allow(unused_imports)]
use super::node_wrappers::{
    AsyncFunctionNodeWrapper, BranchFunctionWrapper, ConfigFunctionNodeWrapper,
    ConfigWriterAsyncFunctionNodeWrapper, FunctionNodeWrapper, RuntimeFunctionNodeWrapper,
};

/// Macro to simplify the creation of workflow nodes.
///
/// This macro provides multiple patterns for creating different types of nodes:
/// - Simple functions
/// - Closures (sync and async)
/// - Config-aware nodes
/// - Nodes with channel writers
/// - Runtime-aware nodes
///
/// # Examples
///
/// ```rust,ignore
/// // Simple function node
/// let node = node!("process", my_function);
///
/// // Closure node
/// let node = node!("transform", |state| {
///     match state {
///         StateValue::String(s) => StateValue::String(s.to_uppercase()),
///         _ => state,
///     }
/// });
///
/// // Async closure node
/// let node = node!("async_process", async |state| {
///     // async processing
///     state
/// });
///
/// // Config-aware node
/// let node = node!("config_node", |state, config| {
///     // use config
///     state
/// });
///
/// // Node with writer
/// let node = node!("writer_node", async |state, config, writer| {
///     // use config and writer
///     state
/// });
///
/// // Runtime-aware node
/// let node = node!("runtime_node", async |state, runtime| {
///     // use runtime
///     state
/// });
/// ```
#[macro_export]
macro_rules! node {
    // standard function mode: <name, function>
    ($name:expr, $func:ident) => {
        Arc::new(FunctionNodeWrapper::new($name.to_string(), $func))
    };

    // closure mode
    ($name:expr, |$state:ident| $body:expr) => {
        Arc::new(FunctionNodeWrapper::new($name.to_string(), |$state| $body))
    };

    ($name:expr, async |$state:ident| $body:expr) => {
        Arc::new(AsyncFunctionNodeWrapper::new(
            $name.to_string(),
            |$state| async move { $body },
        ))
    };

    ($name:expr, |$state:ident, $config:ident| $body:expr) => {
        Arc::new(ConfigFunctionNodeWrapper::new(
            $name.to_string(),
            |$state, $config| $body,
        ))
    };

    ($name:expr, async |$state:ident, $config:ident, $writer:ident| $body:expr) => {
        Arc::new(ConfigWriterAsyncFunctionNodeWrapper::new(
            $name.to_string(),
            |$state, $config, $writer| async { $body },
        ))
    };

    ($name:expr, async |$state:ident, $runtime:ident| $body:expr) => {
        Arc::new(RuntimeFunctionNodeWrapper::new(
            $name.to_string(),
            |$state, $runtime| async { $body },
        ))
    };
}

/// Macro to simplify the creation of branch nodes for conditional routing.
///
/// Branch nodes are used to route workflow execution based on the current state.
/// They return a `BranchResult` indicating the next node(s) to execute.
///
/// # Examples
///
/// ```rust,ignore
/// // Simple conditional routing
/// let branch = branch!("route_by_length", |state| {
///     match state {
///         StateValue::String(s) if s.len() > 10 => {
///             Ok(BranchResult::Single("long_text".to_string()))
///         }
///         _ => Ok(BranchResult::Single("short_text".to_string()))
///     }
/// });
///
/// // Multiple target routing
/// let branch = branch!("route_by_type", |state| {
///     match state {
///         StateValue::String(_) => Ok(BranchResult::Multiple(vec!["string_processor".to_string()])),
///         StateValue::Number(_) => Ok(BranchResult::Multiple(vec!["number_processor".to_string()])),
///         _ => Ok(BranchResult::Single("default_processor".to_string()))
///     }
/// });
///
/// // Send with modified state
/// let branch = branch!("transform_and_route", |state| {
///     let modified_state = StateValue::String("processed".to_string());
///     Ok(BranchResult::Send("next_node".to_string(), modified_state))
/// });
/// ```
#[macro_export]
macro_rules! branch {
    ($name:expr, |$state:ident| $body:expr) => {
        Arc::new(BranchFunctionWrapper::new($name.to_string(), |$state| {
            $body
        }))
    };
}
