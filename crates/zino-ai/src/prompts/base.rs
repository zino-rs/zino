use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// basic prompt value enum to hold either a string or a list of messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptValue {
    String(String),
    Messages(Vec<Message>),
}

impl PromptValue {
    pub fn to_string(&self) -> String {
        match self {
            PromptValue::String(s) => s.clone(),
            PromptValue::Messages(msgs) => {
                msgs.iter()
                    .map(|msg| format!("{}: {}", msg.role.as_str(), msg.content.to_string()))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
    
    pub fn to_messages(&self) -> Vec<Message> {
        match self {
            PromptValue::String(s) => {
                vec![Message {
                    role: Role::User,
                    content: Content::Text(s.clone()),
                }]
            }
            PromptValue::Messages(msgs) => msgs.clone(),
        }
    }
    
    // 新增：检查是否为空
    pub fn is_empty(&self) -> bool {
        match self {
            PromptValue::String(s) => s.is_empty(),
            PromptValue::Messages(msgs) => msgs.is_empty(),
        }
    }
    
    // 新增：获取消息数量
    pub fn message_count(&self) -> usize {
        match self {
            PromptValue::String(_) => 1,
            PromptValue::Messages(msgs) => msgs.len(),
        }
    }
}

// 增强的 BasePromptTemplate trait
pub trait BasePromptTemplate {
    type Output;
    
    /// 同步格式化
    fn format(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError>;
    
    /// 异步格式化 - 使用 Pin<Box<dyn Future>> 来支持不同的 Future 类型
    fn format_async(&self, input: &HashMap<String, String>) -> Pin<Box<dyn Future<Output = Result<Self::Output, TemplateError>> + Send + '_>>;
    
    /// 获取输入变量
    fn get_input_variables(&self) -> Vec<String>;
    
    /// 获取可选变量
    fn get_optional_variables(&self) -> Vec<String> {
        vec![]
    }
    
    /// 获取部分变量
    fn get_partial_variables(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    
    /// 验证输入
    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError>;
    
    /// 创建部分模板
    fn partial(&self, variables: HashMap<String, String>) -> Result<Box<dyn BasePromptTemplate<Output = Self::Output>>, TemplateError> {
        Err(TemplateError::ValidationError("Partial templates not supported".to_string()))
    }
    
    /// 获取模板类型
    fn template_type(&self) -> &'static str;
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Missing required variable '{0}' in template")]
    MissingVariable(String),
    #[error("Template parsing failed: {0}")]
    ParseError(String),
    #[error("Template formatting failed: {0}")]
    FormatError(String),
    #[error("Template validation failed: {0}")]
    ValidationError(String),
    #[error("Serialization/deserialization failed: {0}")]
    SerializationError(String),
    #[error("IO operation failed: {0}")]
    IoError(String),
    #[error("Unsupported template format: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),
}