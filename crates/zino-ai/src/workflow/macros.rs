#[allow(unused_imports)]
use super::node_wrappers::{
    AsyncFunctionNodeWrapper, BranchFunctionWrapper, ConfigFunctionNodeWrapper,
    ConfigWriterAsyncFunctionNodeWrapper, FunctionNodeWrapper, RuntimeFunctionNodeWrapper,
};

/// macro to simplify the creation of a node
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

///Simplies the creation of a branch node
#[macro_export]
macro_rules! branch {
    ($name:expr, |$state:ident| $body:expr) => {
        Arc::new(BranchFunctionWrapper::new($name.to_string(), |$state| {
            $body
        }))
    };
}
