use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// state value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateValue {
    /// string state value
    String(String),
    /// number state value
    Number(f64),
    /// integer state value
    Integer(i64),
    /// boolean state value
    Boolean(bool),
    /// object state value
    Object(HashMap<String, StateValue>),
    /// array state value
    Array(Vec<StateValue>),
    /// null state value
    Null,
}

impl StateValue {
    /// attempt to get as string
    pub fn as_string(&self) -> Option<&String> {
        match self {
            StateValue::String(s) => Some(s),
            _ => None,
        }
    }
    /// attempt to get as number (f64)
    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            StateValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    /// attempt to get as integer (i64)
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            StateValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    /// attempt to get as object
    pub fn as_object(&self) -> Option<&HashMap<String, StateValue>> {
        match self {
            StateValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
    /// attempt to get as array
    pub fn as_array(&self) -> Option<&Vec<StateValue>> {
        match self {
            StateValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

/// channel types
#[derive(Debug, Clone)]
pub enum Channel {
    /// last value channel - stores the last written value
    LastValue(StateValue),
    /// topic channel - supports multiple value accumulation
    Topic(Vec<StateValue>),
    /// ephemeral channel - single-use
    Ephemeral(StateValue),
}

impl Channel {
    /// create a new last value channel
    pub fn new_last_value(value: StateValue) -> Self {
        Channel::LastValue(value)
    }

    /// create a new topic channel
    pub fn new_topic() -> Self {
        Channel::Topic(Vec::new())
    }

    /// create a new ephemeral channel
    pub fn new_ephemeral(value: StateValue) -> Self {
        Channel::Ephemeral(value)
    }

    /// read the current value from the channel
    pub fn read(&self) -> Option<&StateValue> {
        match self {
            Channel::LastValue(v) => Some(v),
            Channel::Topic(v) => v.last(),
            Channel::Ephemeral(v) => Some(v),
        }
    }
    /// write a value to the channel
    pub fn write(&mut self, value: StateValue) {
        match self {
            Channel::LastValue(v) => *v = value,
            Channel::Topic(v) => v.push(value),
            Channel::Ephemeral(v) => *v = value,
        }
    }
    /// check if the channel is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Channel::LastValue(v) => matches!(v, StateValue::Null),
            Channel::Topic(v) => v.is_empty(),
            Channel::Ephemeral(v) => matches!(v, StateValue::Null),
        }
    }
}

/// workflow state
#[derive(Debug, Clone)]
pub struct WorkflowState {
    /// current execution step
    pub step: usize,
    /// channels in the workflow
    pub channels: HashMap<String, Channel>,
    /// completed nodes in the workflow
    pub completed_nodes: Vec<String>,
    /// pending tasks in the workflow
    pub pending_tasks: Vec<ExecutionTask>,
}

/// execution task
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    /// unique task id
    pub id: uuid::Uuid,
    /// node id
    pub node_id: String,
    /// input state value
    pub input: StateValue,
    /// execution step
    pub step: usize,
}

impl WorkflowState {
    /// create a new workflow state
    pub fn new(channels: HashMap<String, Channel>) -> Self {
        Self {
            step: 0,
            channels,
            completed_nodes: Vec::new(),
            pending_tasks: Vec::new(),
        }
    }
    /// advance to the next step
    pub fn advance(&mut self) {
        self.step += 1;
    }
    /// get a channel by name
    pub fn get_channel(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }
    /// get a mutable reference to a channel by name
    pub fn get_channel_mut(&mut self, name: &str) -> Option<&mut Channel> {
        self.channels.get_mut(name)
    }
    /// write a value to a channel
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
