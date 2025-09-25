//! Workflow execution engine for AI-powered data processing pipelines.
//!
//! This module provides a comprehensive framework for building and executing
//! complex workflows with support for state management, error handling,
//! and extensible node types.
//!
//! # Features
//!
//! - **State Management**: Flexible state values with type-safe operations
//! - **Channel Communication**: Inter-node communication via channels
//! - **Node Wrappers**: Easy adaptation of functions into workflow nodes
//! - **Error Handling**: Comprehensive error types with detailed messages
//! - **Macros**: Simplified workflow definition with `node!` and `branch!`
//! - **Extensibility**: Trait-based design for custom node implementations
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use zino_ai::workflow::*;
//! use std::collections::HashMap;
//!
//! // Create a simple workflow node
//! let process_node = node!("process_data", |state| {
//!     // Process the input state
//!     match state {
//!         StateValue::String(s) => StateValue::String(s.to_uppercase()),
//!         _ => state,
//!     }
//! });
//!
//! // Create a branch node for conditional routing
//! let branch_node = branch!("route_data", |state| {
//!     match state {
//!         StateValue::String(s) if s.len() > 10 => {
//!             Ok(BranchResult::Single("long_text".to_string()))
//!         }
//!         _ => Ok(BranchResult::Single("short_text".to_string()))
//!     }
//! });
//! ```
//!
//! # Architecture
//!
//! The workflow system consists of several key components:
//!
//! - **StateValue**: Dynamic value type supporting strings, numbers, objects, and arrays
//! - **Channel**: Communication mechanism between nodes with different semantics
//! - **NodeContext**: Execution context providing configuration and services
//! - **StateNode**: Trait for implementing custom workflow nodes
//! - **BranchPath**: Trait for implementing conditional routing logic
//!
//! # Examples
//!
//! See the individual module documentation for detailed examples of each component.

/// Configuration types for workflow nodes.
pub mod config;

/// Error types for workflow execution.
pub mod error;

/// Macros for simplifying workflow node creation.
pub mod macros;

/// Node wrapper implementations for different function types.
pub mod node_wrappers;

/// State management types for workflow execution.
pub mod state;

/// Core traits for workflow node implementations.
pub mod traits;

/// State graph implementation for workflow execution.
pub mod graph;

/// Workflow executor for running compiled state graphs.
pub mod executor;
