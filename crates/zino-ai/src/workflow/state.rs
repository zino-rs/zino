use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dynamic state value types for workflow execution.
///
/// `StateValue` represents the data that flows through workflow nodes.
/// It supports various data types commonly used in data processing pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateValue {
    /// String state value for text data.
    String(String),
    /// Floating-point number state value.
    Number(f64),
    /// Integer state value.
    Integer(i64),
    /// Boolean state value for true/false values.
    Boolean(bool),
    /// Object state value for key-value pairs.
    Object(HashMap<String, StateValue>),
    /// Array state value for ordered collections.
    Array(Vec<StateValue>),
    /// Null state value representing absence of data.
    Null,
}

impl StateValue {
    /// Attempts to extract the string value if this is a `String` variant.
    ///
    /// Returns `Some(&String)` if the value is a string, `None` otherwise.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            StateValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Attempts to extract the numeric value as `f64`.
    ///
    /// Returns `Some(f64)` if the value is a `Number` or `Integer`, `None` otherwise.
    /// Integer values are automatically converted to floating-point.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            StateValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Attempts to extract the boolean value if this is a `Boolean` variant.
    ///
    /// Returns `Some(bool)` if the value is a boolean, `None` otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            StateValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Attempts to extract the object value if this is an `Object` variant.
    ///
    /// Returns `Some(&HashMap<String, StateValue>)` if the value is an object, `None` otherwise.
    pub fn as_object(&self) -> Option<&HashMap<String, StateValue>> {
        match self {
            StateValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Attempts to extract the array value if this is an `Array` variant.
    ///
    /// Returns `Some(&Vec<StateValue>)` if the value is an array, `None` otherwise.
    pub fn as_array(&self) -> Option<&Vec<StateValue>> {
        match self {
            StateValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

/// Channel types for inter-node communication in workflows.
///
/// Channels provide different communication semantics for workflow nodes:
/// - `LastValue`: Stores only the most recent value (overwrites previous)
/// - `Topic`: Accumulates multiple values (like a message queue)
/// - `Ephemeral`: Single-use channel that is consumed after reading
#[derive(Debug, Clone)]
pub enum Channel {
    /// Last value channel - stores only the most recent written value.
    LastValue(StateValue),
    /// Topic channel - accumulates multiple values for processing.
    Topic(Vec<StateValue>),
    /// Ephemeral channel - single-use channel that is consumed after reading.
    Ephemeral(StateValue),
}

impl Channel {
    /// Creates a new last value channel with the given initial value.
    ///
    /// Last value channels store only the most recent value, overwriting any previous value.
    pub fn new_last_value(value: StateValue) -> Self {
        Channel::LastValue(value)
    }

    /// Creates a new topic channel for accumulating multiple values.
    ///
    /// Topic channels store all written values in order, allowing for batch processing.
    pub fn new_topic() -> Self {
        Channel::Topic(Vec::new())
    }

    /// Creates a new ephemeral channel with the given value.
    ///
    /// Ephemeral channels are single-use and typically consumed after reading.
    pub fn new_ephemeral(value: StateValue) -> Self {
        Channel::Ephemeral(value)
    }

    /// Reads the current value from the channel.
    ///
    /// For `LastValue` and `Ephemeral` channels, returns the stored value.
    /// For `Topic` channels, returns the last written value.
    pub fn read(&self) -> Option<&StateValue> {
        match self {
            Channel::LastValue(v) => Some(v),
            Channel::Topic(v) => v.last(),
            Channel::Ephemeral(v) => Some(v),
        }
    }

    /// Writes a value to the channel.
    ///
    /// For `LastValue` and `Ephemeral` channels, overwrites the existing value.
    /// For `Topic` channels, appends the value to the collection.
    pub fn write(&mut self, value: StateValue) {
        match self {
            Channel::LastValue(v) => *v = value,
            Channel::Topic(v) => v.push(value),
            Channel::Ephemeral(v) => *v = value,
        }
    }

    /// Checks if the channel is empty.
    ///
    /// For `LastValue` and `Ephemeral` channels, returns true if the value is `Null`.
    /// For `Topic` channels, returns true if no values have been written.
    pub fn is_empty(&self) -> bool {
        match self {
            Channel::LastValue(v) => matches!(v, StateValue::Null),
            Channel::Topic(v) => v.is_empty(),
            Channel::Ephemeral(v) => matches!(v, StateValue::Null),
        }
    }
}

/// Represents the current state of a workflow execution.
///
/// `WorkflowState` tracks the execution progress, available channels,
/// completed nodes, and pending tasks in a workflow.
#[derive(Debug, Clone)]
pub struct WorkflowState {
    /// Current execution step number.
    pub step: usize,
    /// Available channels for inter-node communication.
    pub channels: HashMap<String, Channel>,
    /// List of completed node IDs.
    pub completed_nodes: Vec<String>,
    /// Pending tasks waiting for execution.
    pub pending_tasks: Vec<ExecutionTask>,
}

/// Represents a single execution task in a workflow.
///
/// `ExecutionTask` contains all the information needed to execute a node,
/// including the input state and execution context.
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    /// Unique identifier for this task.
    pub id: uuid::Uuid,
    /// ID of the node to execute.
    pub node_id: String,
    /// Input state value for the node.
    pub input: StateValue,
    /// Execution step when this task was created.
    pub step: usize,
}

impl WorkflowState {
    /// Creates a new workflow state with the given channels.
    pub fn new(channels: HashMap<String, Channel>) -> Self {
        Self {
            step: 0,
            channels,
            completed_nodes: Vec::new(),
            pending_tasks: Vec::new(),
        }
    }

    /// Advances the workflow to the next execution step.
    ///
    /// This increments the step counter, which is used to track execution progress
    /// and ensure proper ordering of tasks.
    pub fn advance(&mut self) {
        self.step += 1;
    }

    /// Gets a reference to a channel by name.
    pub fn get_channel(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }

    /// Gets a mutable reference to a channel by name.
    pub fn get_channel_mut(&mut self, name: &str) -> Option<&mut Channel> {
        self.channels.get_mut(name)
    }

    /// Writes a value to a channel by name.
    ///
    /// Returns `Ok(())` if the write was successful, or `Err(WorkflowError::ChannelNotFound)` if the channel doesn't exist.
    pub fn write_to_channel(
        &mut self,
        name: &str,
        value: StateValue,
    ) -> crate::workflow::error::WorkflowResult<()> {
        if let Some(channel) = self.channels.get_mut(name) {
            channel.write(value);
            Ok(())
        } else {
            Err(crate::workflow::error::WorkflowError::ChannelNotFound(
                name.to_string(),
            ))
        }
    }
}
