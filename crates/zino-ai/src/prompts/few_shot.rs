//! Few-shot prompt template implementation.
//!
//! This module provides the `FewShotPromptTemplate` for creating templates
//! that include examples to guide AI model behavior.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::future::ready;
use std::pin::Pin;

/// A template for few-shot learning with examples.
///
/// `FewShotPromptTemplate` creates prompts that include examples to guide
/// AI model behavior. It supports prefix, examples, and suffix sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FewShotPromptTemplate {
    /// List of example data for the few-shot template.
    examples: Vec<HashMap<String, String>>,
    /// Template for formatting each example.
    example_template: StringPromptTemplate,
    /// Optional prefix template.
    prefix: Option<StringPromptTemplate>,
    /// Suffix template for the final prompt.
    suffix: StringPromptTemplate,
    /// Separator string between examples.
    example_separator: String,
    /// Template format for variable interpolation.
    template_format: TemplateFormat,
}

impl FewShotPromptTemplate {
    /// Creates a new few-shot prompt template.
    ///
    /// # Arguments
    /// * `examples` - List of example data for the template
    /// * `example_template` - Template for formatting each example
    /// * `suffix` - Template for the final prompt suffix
    /// * `format` - Template format for variable interpolation
    ///
    /// # Returns
    /// A new `FewShotPromptTemplate` instance.
    pub fn new(
        examples: Vec<HashMap<String, String>>,
        example_template: StringPromptTemplate,
        suffix: StringPromptTemplate,
        format: TemplateFormat,
    ) -> Self {
        Self {
            examples,
            example_template,
            prefix: None,
            suffix,
            example_separator: "\n\n".to_string(),
            template_format: format,
        }
    }

    /// Adds a prefix template to the few-shot template.
    ///
    /// # Arguments
    /// * `prefix` - Template for the prompt prefix
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn with_prefix(mut self, prefix: StringPromptTemplate) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Sets the separator string between examples.
    ///
    /// # Arguments
    /// * `separator` - String to separate examples
    ///
    /// # Returns
    /// Self for method chaining.
    pub fn with_separator(mut self, separator: String) -> Self {
        self.example_separator = separator;
        self
    }
}

impl BasePromptTemplate for FewShotPromptTemplate {
    type Output = PromptValue;

    fn format(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError> {
        let mut parts = Vec::new();

        // 添加前缀
        if let Some(prefix) = &self.prefix {
            let prefix_value = prefix.format(input)?;
            if let PromptValue::String(s) = prefix_value {
                parts.push(s);
            }
        }

        // 添加示例
        for example in &self.examples {
            let example_value = self.example_template.format(example)?;
            if let PromptValue::String(s) = example_value {
                parts.push(s);
            }
        }

        // 添加后缀
        let suffix_value = self.suffix.format(input)?;
        if let PromptValue::String(s) = suffix_value {
            parts.push(s);
        }

        let result = parts.join(&self.example_separator);
        Ok(PromptValue::String(result))
    }

    fn format_async(
        &self,
        input: &HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, TemplateError>> + Send + '_>> {
        Box::pin(ready(self.format(input)))
    }

    fn get_input_variables(&self) -> Vec<String> {
        let mut variables = self.suffix.get_input_variables();
        if let Some(prefix) = &self.prefix {
            variables.extend(prefix.get_input_variables());
        }
        variables.sort();
        variables.dedup();
        variables
    }

    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError> {
        self.suffix.validate_input(input)?;
        if let Some(prefix) = &self.prefix {
            prefix.validate_input(input)?;
        }
        Ok(())
    }

    fn template_type(&self) -> &'static str {
        "few_shot"
    }
}
