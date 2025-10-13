//! Function registry for managing available functions.

use super::error::FunctionResult;
use super::{FunctionError, FunctionExecutor};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing available functions.
///
/// The `FunctionRegistry` provides a centralized way to register, manage,
/// and retrieve function executors that can be called by the AI system.
pub struct FunctionRegistry {
    /// Map of function names to their executors
    functions: HashMap<String, Arc<dyn FunctionExecutor>>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionRegistry {
    /// Create a new function registry.
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a function executor.
    ///
    /// # Arguments
    /// * `name` - Function name
    /// * `executor` - Function executor implementation
    ///
    /// # Returns
    /// * `Result<(), FunctionError>` - Success or error
    pub fn register<E: FunctionExecutor + 'static>(
        &mut self,
        name: &str,
        executor: E,
    ) -> FunctionResult<()> {
        if self.functions.contains_key(name) {
            return Err(FunctionError::Other(format!(
                "Function '{}' is already registered",
                name
            )));
        }

        self.functions.insert(name.to_string(), Arc::new(executor));
        Ok(())
    }

    /// Get a function executor by name.
    ///
    /// # Arguments
    /// * `name` - Function name
    ///
    /// # Returns
    /// * `Option<Arc<dyn FunctionExecutor>>` - Function executor if found
    pub fn get(&self, name: &str) -> Option<Arc<dyn FunctionExecutor>> {
        self.functions.get(name).cloned()
    }

    /// Check if a function is registered.
    ///
    /// # Arguments
    /// * `name` - Function name
    ///
    /// # Returns
    /// * `bool` - True if function is registered
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names.
    ///
    /// # Returns
    /// * `Vec<String>` - List of registered function names
    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get function information.
    ///
    /// # Arguments
    /// * `name` - Function name
    ///
    /// # Returns
    /// * `Option<FunctionInfo>` - Function information if found
    pub fn get_function_info(&self, name: &str) -> Option<FunctionInfo> {
        self.functions.get(name).map(|executor| FunctionInfo {
            name: executor.get_name().to_string(),
            description: executor.get_description().to_string(),
            parameters_schema: executor.get_parameters_schema(),
        })
    }

    /// Get all function information.
    ///
    /// # Returns
    /// * `Vec<FunctionInfo>` - List of all function information
    pub fn get_all_function_info(&self) -> Vec<FunctionInfo> {
        self.functions
            .values()
            .map(|executor| FunctionInfo {
                name: executor.get_name().to_string(),
                description: executor.get_description().to_string(),
                parameters_schema: executor.get_parameters_schema(),
            })
            .collect()
    }

    /// Remove a function from the registry.
    ///
    /// # Arguments
    /// * `name` - Function name
    ///
    /// # Returns
    /// * `bool` - True if function was removed
    pub fn unregister(&mut self, name: &str) -> bool {
        self.functions.remove(name).is_some()
    }

    /// Clear all registered functions.
    pub fn clear(&mut self) {
        self.functions.clear();
    }

    /// Get the number of registered functions.
    ///
    /// # Returns
    /// * `usize` - Number of registered functions
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if the registry is empty.
    ///
    /// # Returns
    /// * `bool` - True if no functions are registered
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("function_count", &self.functions.len())
            .field("function_names", &self.get_function_names())
            .finish()
    }
}

/// Information about a registered function.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Function parameters schema
    pub parameters_schema: serde_json::Value,
}

impl FunctionInfo {
    /// Create new function information.
    pub fn new(name: String, description: String, parameters_schema: serde_json::Value) -> Self {
        Self {
            name,
            description,
            parameters_schema,
        }
    }
}
