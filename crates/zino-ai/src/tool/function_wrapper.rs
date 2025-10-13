//! Function wrapper for converting regular Rust functions to FunctionExecutor.

use super::error::FunctionResult;
use super::{FunctionError, FunctionExecutor};
use serde_json::Value;

/// Base wrapper for regular Rust functions
pub struct FunctionWrapper<F> {
    name: String,
    description: String,
    parameters_schema: Value,
    function: F,
}

impl<F> FunctionWrapper<F> {
    /// Create a new function wrapper
    pub fn new(name: &str, description: &str, parameters_schema: Value, function: F) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters_schema,
            function,
        }
    }
}

// 这些固定参数的包装器已被更灵活的JSON参数处理替代
// 保留简单的无参数函数包装器，因为这是最常用的
/// Wrapper for functions that take no parameters
pub struct NoParamFunctionWrapper<F> {
    name: String,
    description: String,
    function: F,
}

impl<F> NoParamFunctionWrapper<F> {
    /// Create a new no-parameter function wrapper
    pub fn new(name: &str, description: &str, function: F) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            function,
        }
    }
}

// 实现FunctionExecutor for 无参数函数
#[async_trait::async_trait]
impl<F> FunctionExecutor for NoParamFunctionWrapper<F>
where
    F: Fn() -> String + Send + Sync,
{
    async fn execute(&self, _arguments: Value) -> FunctionResult<Value> {
        let result = (self.function)();
        Ok(serde_json::json!({
            "result": result
        }))
    }

    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_description(&self) -> &str {
        &self.description
    }
    fn get_parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }
}

// 单参数和双参数函数已被更灵活的JSON参数处理替代
// 用户可以使用 helpers::json_function 来处理任意参数

// 实现FunctionExecutor for 通用函数包装器
#[async_trait::async_trait]
impl<F> FunctionExecutor for FunctionWrapper<F>
where
    F: Fn(Value) -> Result<Value, FunctionError> + Send + Sync,
{
    async fn execute(&self, arguments: Value) -> FunctionResult<Value> {
        (self.function)(arguments)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_description(&self) -> &str {
        &self.description
    }
    fn get_parameters_schema(&self) -> Value {
        self.parameters_schema.clone()
    }
}

/// Helper functions for creating function wrappers
pub mod helpers {
    use super::*;

    /// Create a wrapper for a no-parameter function
    pub fn no_param<F>(name: &str, description: &str, function: F) -> NoParamFunctionWrapper<F>
    where
        F: Fn() -> String + Send + Sync,
    {
        NoParamFunctionWrapper::new(name, description, function)
    }

    /// Create a wrapper for a single string parameter function (deprecated, use json_function instead)
    pub fn string_param<F>(
        name: &str,
        description: &str,
        function: F,
    ) -> FunctionWrapper<impl Fn(Value) -> Result<Value, FunctionError> + Send + Sync>
    where
        F: Fn(&str) -> String + Send + Sync,
    {
        let wrapper_function = move |args: Value| -> Result<Value, FunctionError> {
            let param = args.get("param").and_then(|v| v.as_str()).ok_or_else(|| {
                FunctionError::InvalidArguments("Missing 'param' parameter".to_string())
            })?;

            let result = function(param);
            Ok(serde_json::json!({
                "param": param,
                "result": result
            }))
        };

        FunctionWrapper::new(
            name,
            description,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "param": {"type": "string", "description": "Input parameter"}
                },
                "required": ["param"]
            }),
            wrapper_function,
        )
    }

    /// Create a wrapper for a two string parameter function (deprecated, use json_function instead)
    pub fn two_string_params<F>(
        name: &str,
        description: &str,
        function: F,
    ) -> FunctionWrapper<impl Fn(Value) -> Result<Value, FunctionError> + Send + Sync>
    where
        F: Fn(&str, &str) -> String + Send + Sync,
    {
        let wrapper_function = move |args: Value| -> Result<Value, FunctionError> {
            let param1 = args.get("param1").and_then(|v| v.as_str()).ok_or_else(|| {
                FunctionError::InvalidArguments("Missing 'param1' parameter".to_string())
            })?;

            let param2 = args.get("param2").and_then(|v| v.as_str()).ok_or_else(|| {
                FunctionError::InvalidArguments("Missing 'param2' parameter".to_string())
            })?;

            let result = function(param1, param2);
            Ok(serde_json::json!({
                "param1": param1,
                "param2": param2,
                "result": result
            }))
        };

        FunctionWrapper::new(
            name,
            description,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string", "description": "First parameter"},
                    "param2": {"type": "string", "description": "Second parameter"}
                },
                "required": ["param1", "param2"]
            }),
            wrapper_function,
        )
    }

    /// Create a wrapper for a function that takes JSON arguments and returns JSON
    pub fn json_function<F>(
        name: &str,
        description: &str,
        parameters_schema: Value,
        function: F,
    ) -> FunctionWrapper<F>
    where
        F: Fn(Value) -> Result<Value, FunctionError> + Send + Sync,
    {
        FunctionWrapper::new(name, description, parameters_schema, function)
    }

    /// Create a wrapper for a function that takes a HashMap of parameters
    pub fn hashmap_function<F>(
        name: &str,
        description: &str,
        parameters_schema: Value,
        function: F,
    ) -> FunctionWrapper<impl Fn(Value) -> Result<Value, FunctionError> + Send + Sync>
    where
        F: Fn(std::collections::HashMap<String, String>) -> Result<String, FunctionError>
            + Send
            + Sync,
    {
        let wrapper_function = move |args: Value| -> Result<Value, FunctionError> {
            let mut params = std::collections::HashMap::new();

            if let Some(obj) = args.as_object() {
                for (key, value) in obj {
                    if let Some(str_val) = value.as_str() {
                        params.insert(key.clone(), str_val.to_string());
                    } else {
                        params.insert(key.clone(), value.to_string());
                    }
                }
            }

            let result = function(params)?;
            Ok(serde_json::Value::String(result))
        };

        FunctionWrapper::new(name, description, parameters_schema, wrapper_function)
    }
}
