//! Chat prompt template implementation.
//!
//! This module provides the `ChatPromptTemplate` for creating multi-turn
//! conversation templates with support for different message roles and formats.

use super::*;
use crate::completions::Message;
use std::collections::HashMap;
use std::future::Future;
use std::future::ready;
use std::pin::Pin;

/// A template for multi-turn conversations with different message roles.
///
/// `ChatPromptTemplate` allows you to create templates that generate multiple
/// messages in a conversation, supporting system, user, assistant, and custom roles.
pub struct ChatPromptTemplate {
    /// List of message templates for the conversation.
    messages: Vec<MessageTemplate>,
    /// Required input variables for all templates.
    input_variables: Vec<String>,
    /// Optional input variables.
    optional_variables: Vec<String>,
    /// Pre-filled variables for partial templates.
    partial_variables: HashMap<String, String>,
}

/// Represents a single message template with a specific role.
#[derive(Debug, Clone)]
pub enum MessageTemplate {
    /// System message template.
    System(StringPromptTemplate),
    /// User message template.
    User(StringPromptTemplate),
    /// Assistant message template.
    Assistant(StringPromptTemplate),
    /// Custom role message template with role name and template.
    Custom(String, StringPromptTemplate), // (role, template)
}

/// Builder for creating `ChatPromptTemplate` instances.
///
/// The builder provides a fluent interface for constructing chat templates
/// with different message roles and formats.
pub struct ChatPromptTemplateBuilder {
    /// List of message templates being built.
    messages: Vec<MessageTemplate>,
}

impl Default for ChatPromptTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatPromptTemplateBuilder {
    /// Creates a new chat prompt template builder.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Adds a system message template to the chat template.
    ///
    /// # Arguments
    /// * `template` - The template string for the system message
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn system(mut self, template: &str) -> Self {
        self.messages.push(MessageTemplate::System(
            StringPromptTemplate::from_template(template),
        ));
        self
    }

    /// Adds a user message template to the chat template.
    ///
    /// # Arguments
    /// * `template` - The template string for the user message
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn user(mut self, template: &str) -> Self {
        self.messages
            .push(MessageTemplate::User(StringPromptTemplate::from_template(
                template,
            )));
        self
    }

    /// Adds an assistant message template to the chat template.
    ///
    /// # Arguments
    /// * `template` - The template string for the assistant message
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn assistant(mut self, template: &str) -> Self {
        self.messages.push(MessageTemplate::Assistant(
            StringPromptTemplate::from_template(template),
        ));
        self
    }

    /// Adds a custom role message template to the chat template.
    ///
    /// # Arguments
    /// * `role` - The role name for the message
    /// * `template` - The template string for the message
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn custom_role(mut self, role: &str, template: &str) -> Self {
        self.messages.push(MessageTemplate::Custom(
            role.to_string(),
            StringPromptTemplate::from_template(template),
        ));
        self
    }

    /// Adds a system message template with the specified format.
    ///
    /// # Arguments
    /// * `template` - The template string for the system message
    /// * `format` - The template format to use
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn system_with_format(mut self, template: &str, format: TemplateFormat) -> Self {
        self.messages.push(MessageTemplate::System(
            StringPromptTemplate::from_template_with_format(template, format),
        ));
        self
    }

    /// Adds a user message template with the specified format.
    ///
    /// # Arguments
    /// * `template` - The template string for the user message
    /// * `format` - The template format to use
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn user_with_format(mut self, template: &str, format: TemplateFormat) -> Self {
        self.messages.push(MessageTemplate::User(
            StringPromptTemplate::from_template_with_format(template, format),
        ));
        self
    }

    /// Adds an assistant message template with the specified format.
    ///
    /// # Arguments
    /// * `template` - The template string for the assistant message
    /// * `format` - The template format to use
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn assistant_with_format(mut self, template: &str, format: TemplateFormat) -> Self {
        self.messages.push(MessageTemplate::Assistant(
            StringPromptTemplate::from_template_with_format(template, format),
        ));
        self
    }

    /// Adds a custom role message template with the specified format.
    ///
    /// # Arguments
    /// * `role` - The role name for the message
    /// * `template` - The template string for the message
    /// * `format` - The template format to use
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn custom_role_with_format(
        mut self,
        role: &str,
        template: &str,
        format: TemplateFormat,
    ) -> Self {
        self.messages.push(MessageTemplate::Custom(
            role.to_string(),
            StringPromptTemplate::from_template_with_format(template, format),
        ));
        self
    }

    /// Builds the final `ChatPromptTemplate` instance.
    ///
    /// # Returns
    /// A new `ChatPromptTemplate` with all configured message templates.
    pub fn build(self) -> ChatPromptTemplate {
        ChatPromptTemplate::new(self.messages)
    }
}

impl ChatPromptTemplate {
    /// Creates a new chat prompt template from a list of message templates.
    ///
    /// # Arguments
    /// * `messages` - List of message templates for the conversation
    ///
    /// # Returns
    /// A new `ChatPromptTemplate` instance with extracted input variables.
    pub fn new(messages: Vec<MessageTemplate>) -> Self {
        let input_variables = Self::extract_input_variables(&messages);
        Self {
            messages,
            input_variables,
            optional_variables: Vec::new(),
            partial_variables: HashMap::new(),
        }
    }

    /// Creates a new builder for constructing chat prompt templates.
    ///
    /// # Returns
    /// A new `ChatPromptTemplateBuilder` instance.
    pub fn builder() -> ChatPromptTemplateBuilder {
        ChatPromptTemplateBuilder::new()
    }

    /// Creates a chat prompt template from role-template pairs.
    ///
    /// # Arguments
    /// * `role_templates` - Vector of (role, template) pairs
    ///
    /// # Returns
    /// * `Ok(ChatPromptTemplate)` - Successfully created template
    /// * `Err(TemplateError)` - If template creation fails
    pub fn from_role_templates(
        role_templates: Vec<(String, String)>,
    ) -> Result<Self, TemplateError> {
        let mut messages = Vec::new();

        for (role, template) in role_templates {
            let prompt = StringPromptTemplate::from_template(&template);
            let message_template = match role.as_str() {
                "system" => MessageTemplate::System(prompt),
                "user" | "human" => MessageTemplate::User(prompt),
                "assistant" | "ai" => MessageTemplate::Assistant(prompt),
                _ => MessageTemplate::Custom(role, prompt),
            };
            messages.push(message_template);
        }

        Ok(Self::new(messages))
    }

    fn extract_input_variables(messages: &[MessageTemplate]) -> Vec<String> {
        let mut variables = Vec::new();
        for message_template in messages {
            match message_template {
                MessageTemplate::System(template) => {
                    variables.extend(template.get_input_variables())
                }
                MessageTemplate::User(template) => variables.extend(template.get_input_variables()),
                MessageTemplate::Assistant(template) => {
                    variables.extend(template.get_input_variables())
                }
                MessageTemplate::Custom(_, template) => {
                    variables.extend(template.get_input_variables())
                }
            }
        }
        variables.sort();
        variables.dedup();
        variables
    }
}

impl BasePromptTemplate for ChatPromptTemplate {
    type Output = PromptValue;

    fn format(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError> {
        self.validate_input(input)?;

        // Merge partial variables with input variables
        let mut merged = self.partial_variables.clone();
        merged.extend(input.clone());

        let mut messages = Vec::new();

        for message_template in &self.messages {
            let message = match message_template {
                MessageTemplate::System(template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => {
                            return Err(TemplateError::FormatError(
                                "Expected string from system template".to_string(),
                            ));
                        }
                    };
                    Message::system(text)
                }
                MessageTemplate::User(template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => {
                            return Err(TemplateError::FormatError(
                                "Expected string from user template".to_string(),
                            ));
                        }
                    };
                    Message::user(text)
                }
                MessageTemplate::Assistant(template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => {
                            return Err(TemplateError::FormatError(
                                "Expected string from assistant template".to_string(),
                            ));
                        }
                    };
                    Message::assistant(text)
                }
                MessageTemplate::Custom(role, template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => {
                            return Err(TemplateError::FormatError(
                                "Expected string from custom template".to_string(),
                            ));
                        }
                    };
                    match role.as_str() {
                        "system" => Message::system(text),
                        "user" => Message::user(text),
                        "assistant" => Message::assistant(text),
                        _ => Message::user(text),
                    }
                }
            };
            messages.push(message);
        }

        Ok(PromptValue::Messages(messages))
    }

    fn format_async(
        &self,
        input: &HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, TemplateError>> + Send + '_>> {
        Box::pin(ready(self.format(input)))
    }

    fn get_input_variables(&self) -> Vec<String> {
        self.input_variables.clone()
    }

    fn get_optional_variables(&self) -> Vec<String> {
        self.optional_variables.clone()
    }

    fn get_partial_variables(&self) -> HashMap<String, String> {
        self.partial_variables.clone()
    }

    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError> {
        // Check for required variables
        let missing: Vec<_> = self
            .input_variables
            .iter()
            .filter(|var| !input.contains_key(*var) && !self.partial_variables.contains_key(*var))
            .collect();

        if !missing.is_empty() {
            return Err(TemplateError::MissingVariable(format!(
                "Missing required variables: {:?}",
                missing
            )));
        }

        Ok(())
    }

    fn template_type(&self) -> &'static str {
        "chat"
    }
}
