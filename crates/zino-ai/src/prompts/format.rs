use minijinja::{Environment, Template};

// template format enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateFormat {
    FString,
    MiniJinja,
    Mustache,
}

// template formatter struct
pub struct TemplateFormatter {
    format: TemplateFormat,
    jinja_env: Option<Environment<'static>>,
}

impl TemplateFormatter {
    pub fn new(format: TemplateFormat) -> Self {
        let jinja_env = match format {
            TemplateFormat::MiniJinja => {
                let mut env = Environment::new();
                env.set_auto_escape_callback(|_name| minijinja::AutoEscape::None);
                Some(env)
            }
            _ => None,
        };
        
        Self {
            format,
            jinja_env,
        }
    }
    
    pub fn format(&self, template: &str, variables: &HashMap<String, String>) -> Result<String, TemplateError> {
        match self.format {
            TemplateFormat::FString => self.format_f_string(template, variables),
            TemplateFormat::MiniJinja => self.format_mini_jinja(template, variables),
            TemplateFormat::Mustache => self.format_mustache(template, variables),
        }
    }
    
    fn format_f_string(&self, template: &str, variables: &HashMap<String, String>) -> Result<String, TemplateError> {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        // Check for unreplaced variables
        if result.contains('{') && result.contains('}') {
            return Err(TemplateError::MissingVariable("Unreplaced variables found".to_string()));
        }
        
        Ok(result)
    }
    
    fn format_mini_jinja(&self, template: &str, variables: &HashMap<String, String>) -> Result<String, TemplateError> {
        let env = self.jinja_env.as_ref().ok_or_else(|| {
            TemplateError::ParseError("MiniJinja environment not initialized".to_string())
        })?;
        
        let tmpl = env.template_from_str(template)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;
        
        let context = minijinja::Value::from_serialize(variables)
            .map_err(|e| TemplateError::FormatError(e.to_string()))?;
        
        tmpl.render(context)
            .map_err(|e| TemplateError::FormatError(e.to_string()))
    }
    
    fn format_mustache(&self, template: &str, variables: &HashMap<String, String>) -> Result<String, TemplateError> {
        // Simple Mustache implementation
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }
}