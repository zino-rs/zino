/// workflow configurations
pub mod config;
/// workflow module
/// defines the error types and core structures for managing workflows
pub mod error;
/// workflow macros for easier workflow definition
pub mod macros;
/// node wrappers to adapt functions into workflow nodes
pub mod node_wrappers;
/// workflow state management
pub mod state;
/// workflow traits for extensibility
pub mod traits;
