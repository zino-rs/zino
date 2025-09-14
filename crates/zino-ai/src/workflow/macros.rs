#[allow(unused_imports)]
use super::{
    node_wrappers::{
        FunctionNodeWrapper, 
        AsyncFunctionNodeWrapper, 
        ConfigFunctionNodeWrapper, 
        ConfigWriterAsyncFunctionNodeWrapper, 
        RuntimeFunctionNodeWrapper,
        BranchFunctionWrapper,
    },
};

/// 宏：简化节点创建
#[macro_export]
macro_rules! node {
    // 最简单的模式：字符串 + 函数名
    ($name:expr, $func:ident) => {
        Arc::new(FunctionNodeWrapper::new($name.to_string(), $func))
    };
    
    // 闭包模式
    ($name:expr, |$state:ident| $body:expr) => {
        Arc::new(FunctionNodeWrapper::new($name.to_string(), |$state| $body))
    };
    
    ($name:expr, async |$state:ident| $body:expr) => {
        Arc::new(AsyncFunctionNodeWrapper::new($name.to_string(), |$state| async move { $body }))
    };
    
    ($name:expr, |$state:ident, $config:ident| $body:expr) => {
        Arc::new(ConfigFunctionNodeWrapper::new($name.to_string(), |$state, $config| $body))
    };
    
    ($name:expr, async |$state:ident, $config:ident, $writer:ident| $body:expr) => {
        Arc::new(ConfigWriterAsyncFunctionNodeWrapper::new($name.to_string(), |$state, $config, $writer| async { $body }))
    };
    
    ($name:expr, async |$state:ident, $runtime:ident| $body:expr) => {
        Arc::new(RuntimeFunctionNodeWrapper::new($name.to_string(), |$state, $runtime| async { $body }))
    };
}

/// 宏：简化分支创建
#[macro_export]
macro_rules! branch {
    ($name:expr, |$state:ident| $body:expr) => {
        Arc::new(BranchFunctionWrapper::new($name.to_string(), |$state| $body))
    };
}
