#[derive(Debug, Clone, Serialize, Deserialize)]
use crate::completions::{Content, Message, Role};
pub struct StringPromptTemplate {
    template: String,
    input_variables: Vec<String>,
    template_format: TemplateFormat,
    formatter: TemplateFormatter,
    partial_variables: HashMap<String, String>,
}

impl StringPromptTemplate {
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
    
    pub fn from_template(template: &str, format: TemplateFormat) -> Self {
        Self::new(template.to_string(), format)
    }
    
    fn extract_variables(template: &str, format: &TemplateFormat) -> Vec<String> {
        match format {
            TemplateFormat::FString => {
                // 提取 {variable} 格式的变量
                let re = regex::Regex::new(r"\{([^}]+)\}").unwrap();
                re.captures_iter(template)
                    .map(|cap| cap[1].to_string())
                    .collect()
            }
            TemplateFormat::MiniJinja => {
                // 提取 {{ variable }} 格式的变量
                let re = regex::Regex::new(r"\{\{\s*([^}]+)\s*\}\}").unwrap();
                re.captures_iter(template)
                    .map(|cap| cap[1].trim().to_string())
                    .collect()
            }
            TemplateFormat::Mustache => {
                // 提取 {{variable}} 格式的变量
                let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
                re.captures_iter(template)
                    .map(|cap| cap[1].trim().to_string())
                    .collect()
            }
        }
    }
    
    pub fn partial(mut self, variables: HashMap<String, String>) -> Self {
        self.partial_variables.extend(variables);
        self.input_variables.retain(|var| !self.partial_variables.contains_key(var));
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
    
    async fn format_async(&self, input: &HashMap<String, String>) -> Result<Self::Output, TemplateError> {
        // 异步版本，可以在这里添加异步处理逻辑
        self.format(input)
    }
    
    fn get_input_variables(&self) -> Vec<String> {
        self.input_variables.clone()
    }
    
    fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), TemplateError> {
        let missing: Vec<String> = self.input_variables
            .iter()
            .filter(|var| !input.contains_key(*var) && !self.partial_variables.contains_key(*var))
            .cloned()
            .collect();
            
        if !missing.is_empty() {
            return Err(TemplateError::MissingVariable(missing.join(", ")));
        }
        
        Ok(())
    }
}