//! Tool execution system for AI applications.
//!
//! This module provides a comprehensive tool execution framework that allows
//! AI models to call external functions and services during conversations.

pub mod error;
pub mod executor;
pub mod function_wrapper;
pub mod macros;
pub mod registry;

// Re-export main types
pub use error::{FunctionError, FunctionResult as FunctionErrorResult};
pub use executor::ToolCallExecutor;
pub use function_wrapper::helpers;
pub use registry::FunctionRegistry;

/// Trait for implementing function executors.
///
/// This trait defines the interface that all function executors must implement
/// to be callable by the AI system.
#[async_trait::async_trait]
pub trait FunctionExecutor: Send + Sync {
    /// Execute the function with the given arguments.
    ///
    /// # Arguments
    /// * `arguments` - Function arguments as JSON value
    ///
    /// # Returns
    /// * `Result<serde_json::Value, FunctionError>` - Function result or error
    async fn execute(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, FunctionError>;

    /// Get the function name.
    fn get_name(&self) -> &str;

    /// Get the function description.
    fn get_description(&self) -> &str;

    /// Get the function parameters schema.
    fn get_parameters_schema(&self) -> serde_json::Value;
}
