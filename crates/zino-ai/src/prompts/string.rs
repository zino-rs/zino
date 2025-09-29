//! String prompt template implementation.
//!
//! This module provides the `StringPromptTemplate` for creating simple text-based
//! prompt templates with variable interpolation support.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::future::ready;
use std::pin::Pin;

/// A simple string-based prompt template.
///
/// `StringPromptTemplate` provides basic template functionality for single-string
/// prompts with variable interpolation. It supports multiple template formats
/// including FString, Mustache, and MiniJinja.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringPromptTemplate {
    /// The template string with variable placeholders.
    template: String,
    /// List of required input variables.
    input_variables: Vec<String>,
    /// The template format to use for interpolation.
    template_format: TemplateFormat,
    /// The formatter instance for this template.
    formatter: TemplateFormatter,
    /// Pre-filled variables for partial templates.
    partial_variables: HashMap<String, String>,
}

impl StringPromptTemplate {
    /// Creates a new string prompt template.
    ///
    /// # Arguments
    /// * `template` - The template string with variable placeholders
    /// * `format` - The template format to use for interpolation
    ///
    /// # Returns
    /// A new `StringPromptTemplate` instance.
    pub fn new(template: String, format: TemplateFormat) -> Self {
        let input_variables = Self::extract_variables(&template, &format);
        let formatter = TemplateFormatter::new(format.clone());

        Self {
            template,
            input_variables,
            template_format: format,
            formatter,
            partial_variables: HashMap::new(),
        }
    }

    /// Creates a new string prompt template with FString format.
    ///
    /// # Arguments
    /// * `template` - The template string with `{variable}` placeholders
    ///
    /// # Returns
    /// A new `StringPromptTemplate` instance using FString format.
    pub fn from_template(template: &str) -> Self {
        Self::new(template.to_string(), TemplateFormat::FString)
    }

    /// Creates a new string prompt template with the specified format.
    ///
    /// # Arguments
    /// * `template` - The template string with variable placeholders
    /// * `format` - The template format to use
    ///
    /// # Returns
    /// A new `StringPromptTemplate` instance with the specified format.
    pub fn from_template_with_format(template: &str, format: TemplateFormat) -> Self {
        Self::new(template.to_string(), format)
    }

    /// Extracts variable names from a template string based on the specified format.
    ///
    /// This method parses the template string and identifies all variable placeholders
    /// according to the template format syntax:
    /// - FString: `{variable}`
    /// - Mustache: `{{variable}}`
    /// - MiniJinja: `{{ variable }}`
    ///
    /// # Arguments
    /// * `template` - The template string to parse
    /// * `format` - The template format to use for parsing
    ///
    /// # Returns
    /// A vector of variable names found in the template.
    fn extract_variables(template: &str, format: &TemplateFormat) -> Vec<String> {
        match format {
            TemplateFormat::FString => {
                // Extract variables in {variable} format - simple implementation
                let mut variables = Vec::new();
                let mut chars = template.chars().peekable();
                let mut current_var = String::new();
                let mut in_var = false;

                #[allow(clippy::while_let_on_iterator)]
                while let Some(ch) = chars.next() {
                    if ch == '{' && chars.peek() == Some(&'}') {
                        // Skip {{
                        chars.next();
                        continue;
                    }
                    if ch == '{' {
                        in_var = true;
                        current_var.clear();
                    } else if ch == '}' && in_var {
                        if !current_var.is_empty() {
                            variables.push(current_var.clone());
                        }
                        in_var = false;
                        current_var.clear();
                    } else if in_var {
                        current_var.push(ch);
                    }
                }
                variables
            }
            TemplateFormat::Mustache => {
                // Extract variables in {{variable}} format - simple implementation
                let mut variables = Vec::new();
                let mut chars = template.chars().peekable();
                let mut current_var = String::new();
                let mut brace_count = 0;

                #[allow(clippy::while_let_on_iterator)]
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        brace_count += 1;
                        if brace_count == 2 {
                            current_var.clear();
                        }
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 && !current_var.is_empty() {
                            variables.push(current_var.trim().to_string());
                            current_var.clear();
                        }
                    } else if brace_count == 2 {
                        current_var.push(ch);
                    }
                }
                variables
            }
            TemplateFormat::MiniJinja => {
                // Extract variables in {{ variable }} format - MiniJinja support
                let mut variables = Vec::new();
                let mut chars = template.chars().peekable();
                let mut current_var = String::new();
                let mut brace_count = 0;

                #[allow(clippy::while_let_on_iterator)]
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        brace_count += 1;
                        if brace_count == 2 {
                            current_var.clear();
                        }
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 && !current_var.is_empty() {
                            variables.push(current_var.trim().to_string());
                            current_var.clear();
                        }
                    } else if brace_count == 2 {
                        current_var.push(ch);
                    }
                }
                variables
            }
        }
    }

    /// Creates a partial template with some variables pre-filled.
    ///
    /// # Arguments
    /// * `variables` - Variables to pre-fill in the template
    ///
    /// # Returns
    /// Self with pre-filled variables, removing them from required inputs.
    pub fn partial(mut self, variables: HashMap<String, String>) -> Self {
        self.partial_variables.extend(variables);
        self.input_variables
            .retain(|var| !self.partial_variables.contains_key(var));
        self
    }
}

impl BasePromptTemplate for StringPromptTemplate {
    type Output = PromptValue;

    fn format(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError> {
        self.validate_input(input)?;

        let mut all_variables = self.partial_variables.clone();
        all_variables.extend(input.clone());

        let formatted = self.formatter.format(&self.template, &all_variables)?;
        Ok(PromptValue::String(formatted))
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

    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError> {
        let missing: Vec<String> = self
            .input_variables
            .iter()
            .filter(|var| !input.contains_key(*var) && !self.partial_variables.contains_key(*var))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return Err(TemplateError::MissingVariable(missing.join(", ")));
        }

        Ok(())
    }

    fn template_type(&self) -> &'static str {
        "string"
    }
}
