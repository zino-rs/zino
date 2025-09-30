//! Tool call executor for processing function calls.

use super::error::FunctionResult;
use super::{FunctionError, FunctionRegistry};
use crate::completions::messages::{Message, ToolCall};
use std::sync::Arc;
use std::time::Duration;

/// Executor for processing tool calls.
///
/// The `ToolCallExecutor` handles the execution of tool calls by routing
/// them to the appropriate function executors registered in the function registry.
#[derive(Debug)]
pub struct ToolCallExecutor {
    /// Function registry containing available functions
    registry: Arc<FunctionRegistry>,
    /// Default timeout for function execution
    default_timeout: Duration,
}

impl ToolCallExecutor {
    /// Create a new tool call executor.
    ///
    /// # Arguments
    /// * `registry` - Function registry containing available functions
    ///
    /// # Returns
    /// * `Self` - New tool call executor
    pub fn new(registry: Arc<FunctionRegistry>) -> Self {
        Self {
            registry,
            default_timeout: Duration::from_secs(30), // 30 seconds default timeout
        }
    }

    /// Create a new tool call executor with custom timeout.
    ///
    /// # Arguments
    /// * `registry` - Function registry containing available functions
    /// * `timeout` - Default timeout for function execution
    ///
    /// # Returns
    /// * `Self` - New tool call executor
    pub fn with_timeout(registry: Arc<FunctionRegistry>, timeout: Duration) -> Self {
        Self {
            registry,
            default_timeout: timeout,
        }
    }

    /// Execute a single tool call.
    ///
    /// # Arguments
    /// * `tool_call` - Tool call to execute
    ///
    /// # Returns
    /// * `Result<String, FunctionError>` - Function result as string
    pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> FunctionResult<String> {
        self.execute_tool_call_with_timeout(tool_call, self.default_timeout)
            .await
    }

    /// Execute a single tool call with custom timeout.
    ///
    /// # Arguments
    /// * `tool_call` - Tool call to execute
    /// * `timeout` - Timeout for function execution
    ///
    /// # Returns
    /// * `Result<String, FunctionError>` - Function result as string
    pub async fn execute_tool_call_with_timeout(
        &self,
        tool_call: &ToolCall,
        timeout: Duration,
    ) -> FunctionResult<String> {
        let function_name = &tool_call.function.name;
        let arguments = &tool_call.function.arguments;

        // Get function executor from registry
        let executor = self
            .registry
            .get(function_name)
            .ok_or_else(|| FunctionError::FunctionNotFound(function_name.clone()))?;

        // Execute function with timeout
        let result = tokio::time::timeout(timeout, executor.execute(arguments.clone()))
            .await
            .map_err(|_| FunctionError::Timeout)?;

        // Convert result to string
        let result_string = match result {
            Ok(value) => {
                if value.is_string() {
                    value.as_str().unwrap_or("").to_string()
                } else {
                    serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
                }
            }
            Err(e) => return Err(e),
        };

        Ok(result_string)
    }

    /// Execute multiple tool calls in parallel.
    ///
    /// # Arguments
    /// * `tool_calls` - Vector of tool calls to execute
    ///
    /// # Returns
    /// * `Result<HashMap<String, String>, FunctionError>` - Results as (tool_call_id, result) pairs
    pub async fn execute_tool_calls_parallel(
        &self,
        tool_calls: &[ToolCall],
    ) -> FunctionResult<std::collections::HashMap<String, String>> {
        let mut handles = Vec::new();

        for tool_call in tool_calls {
            let executor = self.clone();
            let tool_call = tool_call.clone();
            let timeout = self.default_timeout;

            let handle = tokio::spawn(async move {
                let result = executor
                    .execute_tool_call_with_timeout(&tool_call, timeout)
                    .await;
                (tool_call.id, result)
            });

            handles.push(handle);
        }

        let mut results = std::collections::HashMap::new();
        for handle in handles {
            let (tool_call_id, result) = handle
                .await
                .map_err(|e| FunctionError::ExecutionFailed(format!("Task join error: {}", e)))?;

            match result {
                Ok(result_string) => {
                    results.insert(tool_call_id, result_string);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    /// Process messages and execute any tool calls.
    ///
    /// # Arguments
    /// * `messages` - Vector of messages to process
    ///
    /// # Returns
    /// * `Result<Vec<Message>, FunctionError>` - Updated messages with tool results
    pub async fn process_messages(&self, messages: &[Message]) -> FunctionResult<Vec<Message>> {
        let mut result_messages = Vec::new();

        for message in messages {
            match message {
                Message::Assistant { tool_calls, .. } => {
                    if let Some(tool_calls) = tool_calls {
                        // Execute all tool calls in parallel
                        let tool_results = self.execute_tool_calls_parallel(tool_calls).await?;

                        // Add the original assistant message
                        result_messages.push(message.clone());

                        // Add tool result messages
                        for (tool_call_id, result) in tool_results {
                            let tool_message = Message::tool(result, tool_call_id);
                            result_messages.push(tool_message);
                        }
                    } else {
                        result_messages.push(message.clone());
                    }
                }
                _ => result_messages.push(message.clone()),
            }
        }

        Ok(result_messages)
    }

    /// Get function information for a specific function.
    ///
    /// # Arguments
    /// * `function_name` - Name of the function
    ///
    /// # Returns
    /// * `Option<FunctionInfo>` - Function information if found
    pub fn get_function_info(
        &self,
        function_name: &str,
    ) -> Option<crate::tool::registry::FunctionInfo> {
        self.registry.get_function_info(function_name)
    }

    /// Get all available function information.
    ///
    /// # Returns
    /// * `Vec<FunctionInfo>` - List of all function information
    pub fn get_all_function_info(&self) -> Vec<crate::tool::registry::FunctionInfo> {
        self.registry.get_all_function_info()
    }

    /// Check if a function is available.
    ///
    /// # Arguments
    /// * `function_name` - Name of the function
    ///
    /// # Returns
    /// * `bool` - True if function is available
    pub fn is_function_available(&self, function_name: &str) -> bool {
        self.registry.contains(function_name)
    }

    /// Get the default timeout.
    ///
    /// # Returns
    /// * `Duration` - Default timeout
    pub fn get_default_timeout(&self) -> Duration {
        self.default_timeout
    }

    /// Set the default timeout.
    ///
    /// # Arguments
    /// * `timeout` - New default timeout
    pub fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }
}

impl Clone for ToolCallExecutor {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            default_timeout: self.default_timeout,
        }
    }
}
