use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 状态值类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Object(HashMap<String, StateValue>),
    Array(Vec<StateValue>),
    Null,
}

impl StateValue {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            StateValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            StateValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            StateValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    pub fn as_object(&self) -> Option<&HashMap<String, StateValue>> {
        match self {
            StateValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
    
    pub fn as_array(&self) -> Option<&Vec<StateValue>> {
        match self {
            StateValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

/// 通道类型
#[derive(Debug, Clone)]
pub enum Channel {
    /// 最后值通道 - 存储最后写入的值
    LastValue(StateValue),
    /// 主题通道 - 支持多个值累积
    Topic(Vec<StateValue>),
    /// 临时值通道 - 单次使用
    Ephemeral(StateValue),
}

impl Channel {
    pub fn new_last_value(value: StateValue) -> Self {
        Channel::LastValue(value)
    }
    
    pub fn new_topic() -> Self {
        Channel::Topic(Vec::new())
    }
    
    pub fn new_ephemeral(value: StateValue) -> Self {
        Channel::Ephemeral(value)
    }
    
    pub fn read(&self) -> Option<&StateValue> {
        match self {
            Channel::LastValue(v) => Some(v),
            Channel::Topic(v) => v.last(),
            Channel::Ephemeral(v) => Some(v),
        }
    }
    
    pub fn write(&mut self, value: StateValue) {
        match self {
            Channel::LastValue(v) => *v = value,
            Channel::Topic(v) => v.push(value),
            Channel::Ephemeral(v) => *v = value,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        match self {
            Channel::LastValue(v) => matches!(v, StateValue::Null),
            Channel::Topic(v) => v.is_empty(),
            Channel::Ephemeral(v) => matches!(v, StateValue::Null),
        }
    }
}

/// 工作流状态
#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub step: usize,
    pub channels: HashMap<String, Channel>,
    pub completed_nodes: Vec<String>,
    pub pending_tasks: Vec<ExecutionTask>,
}

/// 执行任务
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    pub id: uuid::Uuid,
    pub node_id: String,
    pub input: StateValue,
    pub step: usize,
}

impl WorkflowState {
    pub fn new(channels: HashMap<String, Channel>) -> Self {
        Self {
            step: 0,
            channels,
            completed_nodes: Vec::new(),
            pending_tasks: Vec::new(),
        }
    }
    
    pub fn get_channel(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }
    
    pub fn get_channel_mut(&mut self, name: &str) -> Option<&mut Channel> {
        self.channels.get_mut(name)
    }
    
    pub fn write_to_channel(&mut self, name: &str, value: StateValue) -> crate::workflow::error::WorkflowResult<()> {
        if let Some(channel) = self.channels.get_mut(name) {
            channel.write(value);
            Ok(())
        } else {
            Err(crate::workflow::error::WorkflowError::ChannelNotFound(name.to_string()))
        }
    }
}
