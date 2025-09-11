#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FewShotPromptTemplate {
    examples: Vec<HashMap<String, String>>,
    example_template: StringPromptTemplate,
    prefix: Option<StringPromptTemplate>,
    suffix: StringPromptTemplate,
    example_separator: String,
    template_format: TemplateFormat,
}

impl FewShotPromptTemplate {
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
    
    pub fn with_prefix(mut self, prefix: StringPromptTemplate) -> Self {
        self.prefix = Some(prefix);
        self
    }
    
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
    
    async fn format_async(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError> {
        // 异步版本
        self.format(input)
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
}