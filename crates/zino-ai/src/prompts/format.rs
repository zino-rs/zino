//! Template formatting and format definitions.
//!
//! This module provides support for different template formats including
//! FString, Mustache, and MiniJinja, with a unified formatting interface.

use super::TemplateError;
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported template formats for variable interpolation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateFormat {
    /// FString-style formatting using `{variable}` syntax.
    FString,
    /// MiniJinja templating with `{{ variable }}` syntax and advanced features.
    MiniJinja,
    /// Mustache-style templating using `{{variable}}` syntax.
    Mustache,
}

/// Handles template formatting for different template formats.
///
/// The `TemplateFormatter` provides a unified interface for formatting templates
/// regardless of the underlying template format (FString, Mustache, MiniJinja).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFormatter {
    /// The template format to use for formatting.
    format: TemplateFormat,
    /// MiniJinja environment (only used for MiniJinja format).
    #[serde(skip)]
    jinja_env: Option<Environment<'static>>,
}

impl TemplateFormatter {
    /// Creates a new template formatter for the specified format.
    ///
    /// # Arguments
    /// * `format` - The template format to use
    ///
    /// # Returns
    /// A new `TemplateFormatter` instance configured for the specified format.
    pub fn new(format: TemplateFormat) -> Self {
        let jinja_env = match format {
            TemplateFormat::MiniJinja => {
                let mut env = Environment::new();
                env.set_auto_escape_callback(|_name| minijinja::AutoEscape::None);
                Some(env)
            }
            _ => None,
        };

        Self { format, jinja_env }
    }

    /// Formats a template string with the given variables.
    ///
    /// # Arguments
    /// * `template` - The template string to format
    /// * `variables` - A map of variable names to their values
    ///
    /// # Returns
    /// * `Ok(String)` - The formatted template
    /// * `Err(TemplateError)` - If formatting fails
    pub fn format(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        match self.format {
            TemplateFormat::FString => self.format_f_string(template, variables),
            TemplateFormat::MiniJinja => self.format_mini_jinja(template, variables),
            TemplateFormat::Mustache => self.format_mustache(template, variables),
        }
    }

    fn format_f_string(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Check for unreplaced variables
        if result.contains('{') && result.contains('}') {
            return Err(TemplateError::MissingVariable(
                "Unreplaced variables found".to_string(),
            ));
        }

        Ok(result)
    }

    fn format_mini_jinja(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        let env = self.jinja_env.as_ref().ok_or_else(|| {
            TemplateError::ParseError("MiniJinja environment not initialized".to_string())
        })?;

        let tmpl = env
            .template_from_str(template)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;

        let context = minijinja::Value::from_serialize(variables);

        tmpl.render(context)
            .map_err(|e| TemplateError::FormatError(e.to_string()))
    }

    fn format_mustache(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        // Simple Mustache implementation
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}
