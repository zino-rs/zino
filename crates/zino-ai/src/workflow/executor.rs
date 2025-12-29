//! # Workflow Executor Module.
//!
//! Provides the execution engine for workflows, responsible for runtime execution of state graphs.
//!
//! ## Main Components.
//! - `WorkflowExecutor`: Workflow executor responsible for executing compiled state graphs.
//! - Execution strategies and error handling.
//! - State management and channel communication.
//!
//! ## Usage Example.
//! ```rust
//! use zino_ai::workflow::executor::WorkflowExecutor;
//! use zino_ai::workflow::graph::StateGraph;
//! use std::collections::HashMap;
//! use zino_ai::workflow::state::StateValue;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let graph = StateGraph::new("MyState".to_string());
//! let mut executor = WorkflowExecutor::new(graph, 100)?;
//! let input = HashMap::new();
//! let result = executor.execute(input).await?;
//! # Ok(())
//! # }
//! ```

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{
    error::{WorkflowError, WorkflowResult},
    graph::StateGraph,
    state::{ExecutionTask, StateValue, WorkflowState},
    traits::NodeContext,
};

/// Workflow executor.
///
/// Responsible for executing compiled state graphs, managing node execution order and state transfer.
///
/// # Parameters
/// * `graph` - The state graph to execute.
/// * `max_steps` - Maximum number of execution steps.
///
/// # Example
/// ```rust
/// use zino_ai::workflow::executor::WorkflowExecutor;
/// use zino_ai::workflow::graph::StateGraph;
/// use std::collections::HashMap;
/// use zino_ai::workflow::state::StateValue;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let graph = StateGraph::new("MyState".to_string());
/// let mut executor = WorkflowExecutor::new(graph, 100)?;
/// let input = HashMap::new();
/// let result = executor.execute(input).await?;
/// # Ok(())
/// # }
/// ```
pub struct WorkflowExecutor {
    /// State graph.
    graph: StateGraph,
    /// Workflow execution state.
    state: WorkflowState,
    /// Maximum number of execution steps.
    max_steps: usize,
}

impl WorkflowExecutor {
    /// Create a new workflow executor.
    ///
    /// # Parameters
    /// * `graph` - The state graph to execute.
    /// * `max_steps` - Maximum number of execution steps.
    ///
    /// # Returns
    /// Returns a new `WorkflowExecutor` instance or creation error.
    ///
    /// # Example
    /// ```rust
    /// use zino_ai::workflow::executor::WorkflowExecutor;
    /// use zino_ai::workflow::graph::StateGraph;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let graph = StateGraph::new("MyState".to_string());
    /// let executor = WorkflowExecutor::new(graph, 100)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(graph: StateGraph, max_steps: usize) -> WorkflowResult<Self> {
        Ok(Self {
            state: WorkflowState::new(graph.channels.clone()),
            graph,
            max_steps,
        })
    }

    /// Execute the workflow using a three-phase algorithm similar to Pregel.
    ///
    /// # Parameters
    /// * `input` - Input state values.
    ///
    /// # Returns
    /// Returns execution result or execution error.
    ///
    /// # Example
    /// ```rust
    /// use zino_ai::workflow::executor::WorkflowExecutor;
    /// use zino_ai::workflow::graph::StateGraph;
    /// use std::collections::HashMap;
    /// use zino_ai::workflow::state::StateValue;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let graph = StateGraph::new("MyState".to_string());
    /// let mut executor = WorkflowExecutor::new(graph, 100)?;
    /// let input = HashMap::new();
    /// let result = executor.execute(input).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(
        &mut self,
        input: HashMap<String, StateValue>,
    ) -> WorkflowResult<HashMap<String, StateValue>> {
        // Reset workflow state.
        self.state = WorkflowState::new(self.graph.channels.clone());

        // Initialize input.
        for (channel_name, value) in input {
            self.state.write_to_channel(&channel_name, value)?;
        }

        // Main execution loop.
        while self.state.step < self.max_steps {
            // Phase 1: Plan - determine nodes to execute.
            let tasks_to_execute = self.plan_next_tasks().await?;

            if tasks_to_execute.is_empty() {
                break; // No more tasks, execution complete.
            }

            // Phase 2: Execution - execute all tasks in parallel.
            let results = self.execute_tasks_parallel(tasks_to_execute).await?;

            // Phase 3: Update - update channel values.
            self.update_channels(results).await?;

            self.state.step += 1;
        }

        // Collect outputs.
        self.collect_outputs()
    }

    /// Phase 1: Plan the next tasks to execute.
    ///
    /// # Returns
    /// Returns a vector of tasks ready for execution.
    async fn plan_next_tasks(&mut self) -> WorkflowResult<Vec<ExecutionTask>> {
        let mut tasks = Vec::new();

        for node_id in self.graph.nodes.keys() {
            // Skip completed nodes.
            if self.state.completed_nodes.contains(node_id) {
                continue;
            }

            // Check if trigger conditions are met.
            if self.should_trigger_node(node_id).await? {
                // Read input data.
                let input = self.read_node_input(node_id)?;

                let task = ExecutionTask {
                    id: Uuid::new_v4(),
                    node_id: node_id.clone(),
                    input,
                    step: self.state.step,
                };

                tasks.push(task);
            }
        }

        Ok(tasks)
    }

    /// Check if a node should be triggered.
    ///
    /// # Parameters
    /// * `node_id` - The node ID to check.
    ///
    /// # Returns
    /// Returns `true` if the node should be triggered, `false` otherwise.
    async fn should_trigger_node(&self, node_id: &str) -> WorkflowResult<bool> {
        // Check if node is already completed.
        if self.state.completed_nodes.contains(&node_id.to_string()) {
            return Ok(false);
        }

        // Get all predecessor nodes.
        let predecessors: Vec<&String> = self
            .graph
            .edges
            .iter()
            .filter_map(|(from, to)| if to == node_id { Some(from) } else { None })
            .collect();

        // If no predecessors, it's an entry node, always trigger.
        if predecessors.is_empty() {
            return Ok(true);
        }

        // Check branch logic: if current node is a direct successor of a branch node.
        for predecessor in &predecessors {
            // Check if this predecessor is a branch node.
            if self.is_branch_node(predecessor) {
                // Check branch node output to determine which branch should execute.
                let branch_output_channel = format!("{}_output", predecessor);
                if let Some(channel) = self.state.get_channel(&branch_output_channel) {
                    if let Some(output_value) = channel.read() {
                        if let Some(target_branch) = output_value.as_string() {
                            // If current node is a direct successor of the branch node.
                            let is_direct_successor = self
                                .graph
                                .edges
                                .iter()
                                .any(|(from, to)| from == *predecessor && to == node_id);

                            if is_direct_successor {
                                // Only the selected target branch node can execute.
                                return Ok(node_id == target_branch);
                            }
                            // If current node is not a direct successor of the branch node, continue normal check.
                        }
                    }
                }
                // If branch node has no output, don't allow any node to be triggered.
                return Ok(false);
            }
        }

        // Check predecessor completion status.
        let completed_predecessors: Vec<&String> = predecessors
            .iter()
            .filter(|pred| self.state.completed_nodes.contains(**pred))
            .copied()
            .collect();

        // If no predecessors are completed, don't allow execution.
        if completed_predecessors.is_empty() {
            return Ok(false);
        }

        // Check if there are branch dependencies.
        // Branch dependency: predecessor is a direct successor of a branch node.
        let has_branch_dependency = predecessors.iter().any(|pred| {
            // Check if this predecessor is a direct successor of some branch node.
            self.graph
                .edges
                .iter()
                .any(|(from, to)| to == *pred && self.is_branch_node(from))
        });

        // Debug information.
        #[cfg(debug_assertions)]
        if node_id == "output" || node_id == "large" || node_id == "small" || node_id == "final" {
            eprintln!("🔍 Checking node {} execution conditions:", node_id);
            eprintln!("   Predecessors: {:?}", predecessors);
            eprintln!("   Completed predecessors: {:?}", completed_predecessors);
            eprintln!("   Has branch dependency: {}", has_branch_dependency);
            eprintln!(
                "   Completed nodes: {}, Total predecessors: {}",
                completed_predecessors.len(),
                predecessors.len()
            );
        }

        // If there's a branch dependency, only one predecessor needs to be completed.
        // If there's no branch dependency, all predecessors need to be completed.
        if !has_branch_dependency && completed_predecessors.len() != predecessors.len() {
            #[cfg(debug_assertions)]
            if node_id == "output" || node_id == "large" || node_id == "small" || node_id == "final"
            {
                eprintln!(
                    "   ❌ Blocking node {} execution: all predecessors must be completed",
                    node_id
                );
            }
            return Ok(false);
        }

        Ok(true)
    }

    /// Check if a node is a branch node.
    ///
    /// # Parameters
    /// * `node_id` - The node ID to check.
    ///
    /// # Returns
    /// Returns `true` if the node is a branch node, `false` otherwise.
    fn is_branch_node(&self, node_id: &str) -> bool {
        // Check if this node has multiple direct successor nodes.
        let successors: Vec<&String> = self
            .graph
            .edges
            .iter()
            .filter_map(|(from, to)| if from == node_id { Some(to) } else { None })
            .collect();

        // If there are multiple successors, it's considered a branch node.
        successors.len() > 1
    }

    /// Find the controlling branch node for specified branch successors.
    ///
    /// # Parameters
    /// * `successors` - The successor nodes to find controlling branch for.
    ///
    /// # Returns
    /// Returns the controlling branch node ID if found.
    #[allow(dead_code)]
    fn find_controlling_branch_for_successors(&self, successors: &[&String]) -> Option<String> {
        // Find common parent nodes of these successor nodes.
        let mut parent_counts = HashMap::new();

        for successor in successors {
            let parents: Vec<&String> = self
                .graph
                .edges
                .iter()
                .filter_map(|(from, to)| if to == *successor { Some(from) } else { None })
                .collect();

            for parent in parents {
                *parent_counts.entry(parent).or_insert(0) += 1;
            }
        }

        // Find parent nodes that all successor nodes depend on.
        let controlling_parents: Vec<String> = parent_counts
            .iter()
            .filter(|&(_, &count)| count == successors.len())
            .map(|(parent, _)| (*parent).clone())
            .collect();

        // Select the first controlling parent node (usually only one).
        controlling_parents.first().cloned()
    }

    /// Check if a node depends on branch nodes.
    ///
    /// # Parameters
    /// * `node_id` - The node ID to check.
    ///
    /// # Returns
    /// Returns `true` if the node depends on branch nodes, `false` otherwise.
    #[allow(dead_code)]
    fn is_branch_dependent_node(&self, node_id: &str) -> bool {
        // Check if this node has multiple predecessors and all these predecessors are direct successors of branch nodes.
        let predecessors: Vec<&String> = self
            .graph
            .edges
            .iter()
            .filter_map(|(from, to)| if to == node_id { Some(from) } else { None })
            .collect();

        if predecessors.len() <= 1 {
            return false;
        }

        // Check if all predecessors are direct successors of branch nodes.
        let mut branch_successor_count = 0;
        for predecessor in &predecessors {
            // Check if this predecessor is a direct successor of some branch node.
            let is_branch_successor = self
                .graph
                .edges
                .iter()
                .any(|(from, to)| to == *predecessor && self.is_branch_node(from));

            if is_branch_successor {
                branch_successor_count += 1;
            }
        }

        // If all predecessors are direct successors of branch nodes, consider it a branch dependency.
        branch_successor_count == predecessors.len()
    }

    /// Find the controlling branch node for specified nodes.
    ///
    /// # Parameters
    /// * `node_ids` - The node IDs to find controlling branch for.
    ///
    /// # Returns
    /// Returns the controlling branch node ID if found.
    #[allow(dead_code)]
    fn find_controlling_branch_node(&self, node_ids: &[&String]) -> Option<String> {
        // Find common ancestor nodes of these nodes.
        let mut common_ancestors = HashMap::new();

        for node_id in node_ids {
            let ancestors = self.get_all_ancestors(node_id);
            for ancestor in ancestors {
                *common_ancestors.entry(ancestor).or_insert(0) += 1;
            }
        }

        // Find ancestor nodes that all nodes depend on.
        let controlling_ancestors: Vec<String> = common_ancestors
            .iter()
            .filter(|&(_, &count)| count == node_ids.len())
            .map(|(ancestor, _)| ancestor.clone())
            .collect();

        // Select the closest ancestor node (shortest distance).
        controlling_ancestors
            .iter()
            .min_by_key(|ancestor| self.get_distance_to_nodes(ancestor, node_ids))
            .cloned()
    }

    /// Get all ancestor nodes for a given node.
    ///
    /// # Parameters
    /// * `node_id` - The node ID to get ancestors for.
    ///
    /// # Returns
    /// Returns a vector of ancestor node IDs.
    #[allow(dead_code)]
    fn get_all_ancestors(&self, node_id: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut to_visit = vec![node_id.to_string()];
        let mut visited = HashSet::new();

        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            let direct_ancestors: Vec<String> = self
                .graph
                .edges
                .iter()
                .filter_map(|(from, to)| {
                    if to == &current {
                        Some(from.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for ancestor in direct_ancestors {
                if !visited.contains(&ancestor) {
                    ancestors.push(ancestor.clone());
                    to_visit.push(ancestor);
                }
            }
        }

        ancestors
    }

    /// Calculate distance from a node to a set of target nodes.
    ///
    /// # Parameters
    /// * `from_node` - The source node.
    /// * `to_nodes` - The target nodes.
    ///
    /// # Returns
    /// Returns the minimum distance to any target node.
    #[allow(dead_code)]
    fn get_distance_to_nodes(&self, from_node: &str, to_nodes: &[&String]) -> usize {
        let mut min_distance = usize::MAX;

        for to_node in to_nodes {
            let distance = self.get_shortest_path_length(from_node, to_node);
            min_distance = min_distance.min(distance);
        }

        min_distance
    }

    /// Calculate the shortest path length between two nodes.
    ///
    /// # Parameters
    /// * `from` - The source node.
    /// * `to` - The target node.
    ///
    /// # Returns
    /// Returns the shortest path length or `usize::MAX` if no path exists.
    #[allow(dead_code)]
    fn get_shortest_path_length(&self, from: &str, to: &str) -> usize {
        if from == to {
            return 0;
        }

        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((from.to_string(), 0));
        visited.insert(from.to_string());

        while let Some((current, distance)) = queue.pop_front() {
            let neighbors: Vec<String> = self
                .graph
                .edges
                .iter()
                .filter_map(|(from_edge, to_edge)| {
                    if from_edge == &current {
                        Some(to_edge.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for neighbor in neighbors {
                if neighbor == to {
                    return distance + 1;
                }

                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back((neighbor, distance + 1));
                }
            }
        }

        usize::MAX // No path found.
    }

    /// Read node input data.
    ///
    /// # Parameters
    /// * `node_id` - The node ID to read input for.
    ///
    /// # Returns
    /// Returns the input state value or error.
    fn read_node_input(&self, node_id: &str) -> WorkflowResult<StateValue> {
        // Get all predecessor nodes.
        let predecessors: Vec<&String> = self
            .graph
            .edges
            .iter()
            .filter_map(|(from, to)| if to == node_id { Some(from) } else { None })
            .collect();

        if !predecessors.is_empty() {
            for predecessor in &predecessors {
                if self.is_branch_node(predecessor) {
                    let branch_input_predecessors: Vec<&String> = self
                        .graph
                        .edges
                        .iter()
                        .filter_map(|(from, to)| if to == *predecessor { Some(from) } else { None })
                        .collect();

                    // Prioritize reading data from completed predecessor nodes.
                    for branch_input_predecessor in &branch_input_predecessors {
                        if self
                            .state
                            .completed_nodes
                            .contains(*branch_input_predecessor)
                        {
                            let output_channel = format!("{}_output", branch_input_predecessor);
                            if let Some(channel) = self.state.get_channel(&output_channel) {
                                if let Some(value) = channel.read() {
                                    // If current node is a result node (usually ending with success/error/result),
                                    // and branch input is boolean type (usually validation result), try to find earlier data source.
                                    if (node_id.contains("success")
                                        || node_id.contains("error")
                                        || node_id.contains("result"))
                                        && matches!(value, StateValue::Boolean(_))
                                    {
                                        // Try to find earlier data source.
                                        let earlier_predecessors: Vec<&String> = self
                                            .graph
                                            .edges
                                            .iter()
                                            .filter_map(|(from, to)| {
                                                if to == *branch_input_predecessor {
                                                    Some(from)
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();

                                        for earlier_predecessor in &earlier_predecessors {
                                            if self
                                                .state
                                                .completed_nodes
                                                .contains(*earlier_predecessor)
                                            {
                                                let earlier_output_channel =
                                                    format!("{}_output", earlier_predecessor);
                                                if let Some(earlier_channel) =
                                                    self.state.get_channel(&earlier_output_channel)
                                                {
                                                    if let Some(earlier_value) =
                                                        earlier_channel.read()
                                                    {
                                                        if !matches!(
                                                            earlier_value,
                                                            StateValue::Boolean(_)
                                                        ) {
                                                            return Ok(earlier_value.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    return Ok(value.clone());
                                }
                            }
                        }
                    }

                    // If no completed predecessor nodes, try to read from the first predecessor with data.
                    for branch_input_predecessor in &branch_input_predecessors {
                        let output_channel = format!("{}_output", branch_input_predecessor);
                        if let Some(channel) = self.state.get_channel(&output_channel)
                            && let Some(value) = channel.read()
                            && !matches!(value, StateValue::Null)
                        {
                            return Ok(value.clone());
                        }
                    }
                }
            }

            // Prioritize reading output from completed predecessor nodes.
            for predecessor in &predecessors {
                if self.state.completed_nodes.contains(*predecessor) {
                    let output_channel = format!("{}_output", predecessor);
                    if let Some(channel) = self.state.get_channel(&output_channel)
                        && let Some(value) = channel.read()
                    {
                        return Ok(value.clone());
                    }
                }
            }

            // If no completed predecessor nodes, try to read from the first predecessor's output.
            if let Some(first_predecessor) = predecessors.first() {
                let output_channel = format!("{}_output", first_predecessor);
                if let Some(channel) = self.state.get_channel(&output_channel)
                    && let Some(value) = channel.read()
                {
                    return Ok(value.clone());
                }
            }
        }

        // If no predecessor nodes, try to read from all available input channels.
        for (channel_name, channel) in &self.state.channels {
            // Skip output channels.
            if channel_name.ends_with("_output") {
                continue;
            }

            if let Some(value) = channel.read()
                && !matches!(value, StateValue::Null)
            {
                return Ok(value.clone());
            }
        }

        Ok(StateValue::Null)
    }

    /// Phase 2: Execute tasks in parallel.
    ///
    /// # Parameters
    /// * `tasks` - Vector of tasks to execute.
    ///
    /// # Returns
    /// Returns execution results or execution error.
    async fn execute_tasks_parallel(
        &self,
        tasks: Vec<ExecutionTask>,
    ) -> WorkflowResult<Vec<(String, StateValue)>> {
        let mut handles = Vec::new();

        for task in tasks {
            let node_spec = self.graph.nodes.get(&task.node_id).ok_or_else(|| {
                WorkflowError::InvalidState(format!("Node {} not found", task.node_id))
            })?;

            let executor = node_spec.runnable.clone();
            let input = task.input.clone();
            let node_id = task.node_id.clone();

            let handle = tokio::spawn(async move {
                let context = NodeContext::new();
                let result = executor.execute(input, &context).await;
                (node_id, result)
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            let (node_id, result) = handle.await.map_err(|e| {
                WorkflowError::NodeExecutionFailed(format!("Task join error: {}", e))
            })?;

            match result {
                Ok(output) => results.push((node_id, output)),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    /// Phase 3: Update channel values.
    ///
    /// # Parameters
    /// * `results` - Execution results to update channels with.
    ///
    /// # Returns
    /// Returns success or error.
    async fn update_channels(&mut self, results: Vec<(String, StateValue)>) -> WorkflowResult<()> {
        for (node_id, output) in results {
            // Write to node's output channel.
            let output_channel = format!("{}_output", node_id);
            self.state.write_to_channel(&output_channel, output)?;

            // Mark node as completed.
            self.state.completed_nodes.push(node_id);
        }

        Ok(())
    }

    /// Collect output results.
    ///
    /// # Returns
    /// Returns a map of output values or collection error.
    fn collect_outputs(&self) -> WorkflowResult<HashMap<String, StateValue>> {
        let mut outputs = HashMap::new();

        // Prioritize collecting final results from end node's output channel.
        let end_output_channel = format!("{}_output", crate::workflow::graph::END_NODE);
        if let Some(channel) = self.state.get_channel(&end_output_channel)
            && let Some(value) = channel.read()
            && !matches!(value, StateValue::Null)
        {
            outputs.insert("final_result".to_string(), value.clone());
        }

        // Collect all other non-null channel values.
        for (name, channel) in &self.state.channels {
            if name != &end_output_channel
                && let Some(value) = channel.read()
                && !matches!(value, StateValue::Null)
            {
                outputs.insert(name.clone(), value.clone());
            }
        }

        Ok(outputs)
    }
}
