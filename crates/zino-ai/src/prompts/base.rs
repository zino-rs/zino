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
}

// Basic trait for prompt templates
pub trait BasePromptTemplate {
    type Output;
    
    fn format(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError>;
    fn format_async(&self, input: &HashMap<String, String>) -> impl std::future::Future<Output = Result<Self::Output, TemplateError>>;
    fn get_input_variables(&self) -> Vec<String>;
    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError>;
}

// Error type for template processing
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Missing required variable: {0}")]
    MissingVariable(String),
    #[error("Template parsing error: {0}")]
    ParseError(String),
    #[error("Format error: {0}")]
    FormatError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}