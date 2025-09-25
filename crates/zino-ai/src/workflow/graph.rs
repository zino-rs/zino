//! # Workflow Graph Module.
//!
//! Provides state graph construction, compilation and execution functionality.
//!
//! ## Main Components.
//! - `StateGraph`: State graph builder.
//! - `CompiledStateGraph`: Compiled executable graph.
//! - `StateNodeSpec`: Node specification definition.
//! - `BranchSpec`: Branch specification definition.
//!
//! ## Usage Example.
//! ```rust
//! use zino_ai::workflow::graph::{StateGraph, StateNodeSpec};
//! use std::collections::HashMap;
//! use zino_ai::workflow::state::StateValue;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut graph = StateGraph::new("MyState".to_string());
//! // ... add nodes and edges
//! let mut compiled = graph.compile()?;
//! let input = HashMap::new();
//! let result = compiled.invoke(input).await?;
//! # Ok(())
//! # }
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{
    config::{CachePolicy, RetryPolicy},
    error::WorkflowResult,
    node_wrappers::FunctionNodeWrapper,
    state::{Channel, StateValue},
    traits::{BranchPath, StateNode},
};

/// Workflow constant node names.
pub const START_NODE: &str = "__start__";
/// End node constant for workflow graphs.
pub const END_NODE: &str = "__end__";

/// Node specification.
#[derive(Clone)]
pub struct StateNodeSpec {
    /// Node executor.
    pub runnable: Arc<dyn StateNode>,
    /// Metadata.
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Retry policy.
    pub retry_policy: Option<RetryPolicy>,
    /// Cache policy.
    pub cache_policy: Option<CachePolicy>,
    /// Whether to defer execution.
    pub defer: bool,
}

impl std::fmt::Debug for StateNodeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateNodeSpec")
            .field("runnable", &"<dyn StateNode>")
            .field("metadata", &self.metadata)
            .field("retry_policy", &self.retry_policy)
            .field("cache_policy", &self.cache_policy)
            .field("defer", &self.defer)
            .finish()
    }
}

impl StateNodeSpec {
    /// Create a new node specification.
    ///
    /// # Parameters
    /// * `runnable` - The node executor.
    ///
    /// # Returns
    /// Returns a new `StateNodeSpec` instance.
    pub fn new(runnable: Arc<dyn StateNode>) -> Self {
        Self {
            runnable,
            metadata: None,
            retry_policy: None,
            cache_policy: None,
            defer: false,
        }
    }

    /// Set metadata for the node.
    ///
    /// # Parameters
    /// * `metadata` - Node metadata.
    ///
    /// # Returns
    /// Returns `Self` for method chaining.
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set retry policy for the node.
    ///
    /// # Parameters
    /// * `policy` - Retry policy.
    ///
    /// # Returns
    /// Returns `Self` for method chaining.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Set cache policy for the node.
    ///
    /// # Parameters
    /// * `policy` - Cache policy.
    ///
    /// # Returns
    /// Returns `Self` for method chaining.
    pub fn with_cache_policy(mut self, policy: CachePolicy) -> Self {
        self.cache_policy = Some(policy);
        self
    }

    /// Set defer flag for the node.
    ///
    /// # Parameters
    /// * `defer` - Whether to defer execution.
    ///
    /// # Returns
    /// Returns `Self` for method chaining.
    pub fn with_defer(mut self, defer: bool) -> Self {
        self.defer = defer;
        self
    }
}

/// Branch specification.
#[derive(Clone)]
pub struct BranchSpec {
    /// Path function.
    pub path: Arc<dyn BranchPath>,
    /// End point mapping.
    pub ends: Option<HashMap<String, String>>,
    /// Input schema.
    pub input_schema: Option<String>,
}

impl std::fmt::Debug for BranchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BranchSpec")
            .field("path", &"<dyn BranchPath>")
            .field("ends", &self.ends)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

impl BranchSpec {
    /// Create a new branch specification.
    ///
    /// # Parameters
    /// * `path` - The path function.
    ///
    /// # Returns
    /// Returns a new `BranchSpec` instance.
    pub fn new(path: Arc<dyn BranchPath>) -> Self {
        Self {
            path,
            ends: None,
            input_schema: None,
        }
    }

    /// Set end point mapping for the branch.
    ///
    /// # Parameters
    /// * `ends` - End point mapping.
    ///
    /// # Returns
    /// Returns `Self` for method chaining.
    pub fn with_ends(mut self, ends: HashMap<String, String>) -> Self {
        self.ends = Some(ends);
        self
    }

    /// Set input schema for the branch.
    ///
    /// # Parameters
    /// * `schema` - Input schema.
    ///
    /// # Returns
    /// Returns `Self` for method chaining.
    pub fn with_input_schema(mut self, schema: String) -> Self {
        self.input_schema = Some(schema);
        self
    }
}

/// State graph.
#[derive(Debug, Clone)]
pub struct StateGraph {
    /// Node mapping.
    pub nodes: HashMap<String, StateNodeSpec>,
    /// Edge set.
    pub edges: HashSet<(String, String)>,
    /// Branch mapping.
    pub branches: HashMap<String, HashMap<String, BranchSpec>>,
    /// Channel mapping.
    pub channels: HashMap<String, Channel>,
    /// Waiting edges.
    pub waiting_edges: HashSet<(Vec<String>, String)>,
    /// Whether compiled.
    pub compiled: bool,
    /// State schema.
    pub state_schema: String,
    /// Context schema.
    pub context_schema: Option<String>,
    /// Input schema.
    pub input_schema: String,
    /// Output schema.
    pub output_schema: String,
}

impl StateGraph {
    /// Create a new state graph.
    ///
    /// # Parameters
    /// * `state_schema` - State schema definition.
    ///
    /// # Returns
    /// Returns a new `StateGraph` instance.
    ///
    /// # Example
    /// ```rust
    /// use zino_ai::workflow::graph::StateGraph;
    ///
    /// let graph = StateGraph::new("MyState".to_string());
    /// ```
    pub fn new(state_schema: String) -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            edges: HashSet::new(),
            branches: HashMap::new(),
            channels: HashMap::new(),
            waiting_edges: HashSet::new(),
            compiled: false,
            state_schema,
            context_schema: None,
            input_schema: "State".to_string(),
            output_schema: "State".to_string(),
        };

        // Automatically add start and end nodes.
        graph.add_start_end_nodes();
        graph
    }

    /// Add start and end nodes.
    fn add_start_end_nodes(&mut self) {
        // Add start node - just pass through input data.
        let start_node = StateNodeSpec::new(Arc::new(FunctionNodeWrapper::new(
            START_NODE.to_string(),
            Ok, // Directly pass through input.
        )));

        // Add end node - collect final result.
        let end_node = StateNodeSpec::new(Arc::new(FunctionNodeWrapper::new(
            END_NODE.to_string(),
            Ok, // Directly pass through input as final result.
        )));

        self.nodes.insert(START_NODE.to_string(), start_node);
        self.nodes.insert(END_NODE.to_string(), end_node);
    }

    /// Add a node to the graph.
    ///
    /// # Parameters
    /// * `name` - Node name.
    /// * `node_spec` - Node specification.
    ///
    /// # Returns
    /// Returns `&mut Self` for method chaining.
    pub fn add_node(&mut self, name: String, node_spec: StateNodeSpec) -> &mut Self {
        self.nodes.insert(name, node_spec);
        self
    }

    /// Add an edge to the graph.
    ///
    /// # Parameters
    /// * `start` - Start node name.
    /// * `end` - End node name.
    ///
    /// # Returns
    /// Returns `&mut Self` for method chaining.
    pub fn add_edge(&mut self, start: String, end: String) -> &mut Self {
        self.edges.insert((start, end));
        self
    }

    /// Add conditional edges to the graph.
    ///
    /// # Parameters
    /// * `start` - Start node name.
    /// * `branch_spec` - Branch specification.
    ///
    /// # Returns
    /// Returns `&mut Self` for method chaining.
    pub fn add_conditional_edges(&mut self, start: String, branch_spec: BranchSpec) -> &mut Self {
        let branch_map = self.branches.entry(start).or_default();
        branch_map.insert("default".to_string(), branch_spec);
        self
    }

    /// Set entry point for the graph.
    ///
    /// # Parameters
    /// * `node` - Entry node name.
    ///
    /// # Returns
    /// Returns `&mut Self` for method chaining.
    pub fn set_entry_point(&mut self, node: String) -> &mut Self {
        self.add_edge(START_NODE.to_string(), node);
        self
    }

    /// Set finish point for the graph.
    ///
    /// # Parameters
    /// * `node` - Finish node name.
    ///
    /// # Returns
    /// Returns `&mut Self` for method chaining.
    pub fn set_finish_point(&mut self, node: String) -> &mut Self {
        self.add_edge(node, END_NODE.to_string());
        self
    }

    /// Compile the graph.
    /// Automatically create node output channels.
    pub fn auto_create_output_channels(&mut self) {
        for node_id in self.nodes.keys() {
            let output_channel = format!("{}_output", node_id);
            self.channels
                .entry(output_channel)
                .or_insert_with(|| Channel::new_last_value(StateValue::Null));
        }
    }

    /// Compile the state graph.
    ///
    /// # Returns
    /// Returns a compiled state graph or compilation error.
    ///
    /// # Example
    /// ```rust
    /// use zino_ai::workflow::graph::StateGraph;
    /// use std::collections::HashMap;
    /// use zino_ai::workflow::state::StateValue;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let graph = StateGraph::new("MyState".to_string());
    /// let mut compiled = graph.compile()?;
    /// let input = HashMap::new();
    /// let result = compiled.invoke(input).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn compile(mut self) -> WorkflowResult<CompiledStateGraph> {
        // Automatically create output channels for all nodes.
        self.auto_create_output_channels();
        self.compiled = true;
        CompiledStateGraph::new(self)
    }
}

/// Compiled state graph.
pub struct CompiledStateGraph {
    #[allow(dead_code)]
    graph: StateGraph,
    executor: crate::workflow::executor::WorkflowExecutor,
}

impl CompiledStateGraph {
    /// Create a new compiled state graph.
    ///
    /// # Parameters
    /// * `graph` - The state graph to compile.
    ///
    /// # Returns
    /// Returns a compiled state graph or compilation error.
    pub fn new(graph: StateGraph) -> WorkflowResult<Self> {
        let executor = crate::workflow::executor::WorkflowExecutor::new(graph.clone(), 100)?;

        Ok(Self { graph, executor })
    }

    /// Invoke the compiled state graph.
    ///
    /// # Parameters
    /// * `input` - Input state values.
    ///
    /// # Returns
    /// Returns execution result or execution error.
    ///
    /// # Example
    /// ```rust
    /// use zino_ai::workflow::graph::{StateGraph, CompiledStateGraph};
    /// use std::collections::HashMap;
    /// use zino_ai::workflow::state::StateValue;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let graph = StateGraph::new("MyState".to_string());
    /// let mut compiled = graph.compile()?;
    /// let input = HashMap::new();
    /// let result = compiled.invoke(input).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn invoke(
        &mut self,
        input: HashMap<String, StateValue>,
    ) -> WorkflowResult<HashMap<String, StateValue>> {
        self.executor.execute(input).await
    }
}
