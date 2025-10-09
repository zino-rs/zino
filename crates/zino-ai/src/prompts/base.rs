//! Base prompt template types and traits.
//!
//! This module provides the foundational types and traits for the prompt template system,
//! including `PromptValue` for representing template outputs and `BasePromptTemplate`
//! trait for defining the common interface for all prompt templates.

use crate::completions::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Represents the output of a prompt template.
///
/// `PromptValue` can hold either a single string or a list of messages,
/// providing flexibility for different template use cases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptValue {
    /// A single string output from the template.
    String(String),
    /// A list of messages output from the template.
    Messages(Vec<Message>),
}

impl fmt::Display for PromptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptValue::String(s) => write!(f, "{}", s),
            PromptValue::Messages(msgs) => {
                let formatted: Vec<String> = msgs
                    .iter()
                    .map(|msg| format!("{}: {}", msg.role(), msg.content().as_string()))
                    .collect();
                write!(f, "{}", formatted.join("\n"))
            }
        }
    }
}

impl PromptValue {
    /// Converts the prompt value to a list of messages.
    ///
    /// For string values, wraps the string in a user message.
    /// For message values, returns the messages directly.
    pub fn to_messages(&self) -> Vec<crate::completions::Message> {
        match self {
            PromptValue::String(s) => {
                vec![Message::user(s.clone())]
            }
            PromptValue::Messages(msgs) => msgs.clone(),
        }
    }

    /// Checks if the prompt value is empty.
    ///
    /// Returns `true` if the string is empty or the message list is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            PromptValue::String(s) => s.is_empty(),
            PromptValue::Messages(msgs) => msgs.is_empty(),
        }
    }

    /// Returns the number of messages in the prompt value.
    ///
    /// For string values, returns 1.
    /// For message values, returns the length of the message list.
    pub fn message_count(&self) -> usize {
        match self {
            PromptValue::String(_) => 1,
            PromptValue::Messages(msgs) => msgs.len(),
        }
    }
}

/// Base trait for all prompt templates.
///
/// This trait defines the common interface that all prompt templates must implement,
/// providing methods for formatting, validation, and introspection.
pub trait BasePromptTemplate {
    /// The output type of the template.
    type Output;

    /// Formats the template with the given input variables.
    ///
    /// # Arguments
    /// * `input` - A map of variable names to their values
    ///
    /// # Returns
    /// * `Ok(output)` - The formatted template output
    /// * `Err(TemplateError)` - If formatting fails
    fn format(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError>;

    /// Asynchronously formats the template with the given input variables.
    ///
    /// This method allows for async operations during template formatting,
    /// such as loading external data or making API calls.
    ///
    /// # Arguments
    /// * `input` - A map of variable names to their values
    ///
    /// # Returns
    /// * `Pin<Box<dyn Future<Output = Result<Self::Output, TemplateError>> + Send + '_>>` - A future that resolves to the formatted output
    fn format_async(
        &self,
        input: &HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, TemplateError>> + Send + '_>>;

    /// Returns the list of required input variables for this template.
    ///
    /// These variables must be provided when formatting the template.
    fn get_input_variables(&self) -> Vec<String>;

    /// Returns the list of optional input variables for this template.
    ///
    /// These variables may be provided but are not required for formatting.
    fn get_optional_variables(&self) -> Vec<String> {
        vec![]
    }

    /// Returns the partial variables that have been pre-filled in this template.
    ///
    /// These variables are already set and don't need to be provided during formatting.
    fn get_partial_variables(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Validates that all required input variables are provided.
    ///
    /// # Arguments
    /// * `input` - The input variables to validate
    ///
    /// # Returns
    /// * `Ok(())` - If all required variables are present
    /// * `Err(TemplateError)` - If any required variables are missing
    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError>;

    /// Creates a partial template with some variables pre-filled.
    ///
    /// # Arguments
    /// * `variables` - Variables to pre-fill in the template
    ///
    /// # Returns
    /// * `Ok(Box<dyn BasePromptTemplate>)` - A new template with pre-filled variables
    /// * `Err(TemplateError)` - If partial template creation is not supported
    fn partial(
        &self,
        _variables: HashMap<String, String>,
    ) -> Result<Box<dyn BasePromptTemplate<Output = Self::Output>>, TemplateError> {
        Err(TemplateError::ValidationError(
            "Partial templates not supported".to_string(),
        ))
    }

    /// Returns the type identifier for this template.
    ///
    /// This is used for introspection and debugging purposes.
    fn template_type(&self) -> &'static str;
}

/// Errors that can occur during template operations.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// A required variable is missing from the input.
    #[error("Missing required variable '{0}' in template")]
    MissingVariable(String),
    /// Template parsing failed due to syntax errors.
    #[error("Template parsing failed: {0}")]
    ParseError(String),
    /// Template formatting failed during execution.
    #[error("Template formatting failed: {0}")]
    FormatError(String),
    /// Template validation failed.
    #[error("Template validation failed: {0}")]
    ValidationError(String),
    /// Serialization or deserialization failed.
    #[error("Serialization/deserialization failed: {0}")]
    SerializationError(String),
    /// IO operation failed.
    #[error("IO operation failed: {0}")]
    IoError(String),
    /// The template format is not supported.
    #[error("Unsupported template format: {0}")]
    UnsupportedFormat(String),
    /// The template syntax is invalid.
    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),
}
