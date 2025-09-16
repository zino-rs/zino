// chat_template.rs
use super::*;
use std::future::ready;

pub struct ChatPromptTemplate {
    messages: Vec<MessageTemplate>,
    input_variables: Vec<String>,
    optional_variables: Vec<String>,
    partial_variables: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum MessageTemplate {
    System(StringPromptTemplate),
    User(StringPromptTemplate),
    Assistant(StringPromptTemplate),
    Custom(String, StringPromptTemplate), // (role, template)
}

/// ChatPromptTemplate 构建器
pub struct ChatPromptTemplateBuilder {
    messages: Vec<MessageTemplate>,
}

impl ChatPromptTemplateBuilder {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
    
    /// 添加系统消息模板
    pub fn system(mut self, template: &str) -> Self {
        self.messages.push(MessageTemplate::System(
            StringPromptTemplate::from_template(template)
        ));
        self
    }
    
    /// 添加用户消息模板
    pub fn user(mut self, template: &str) -> Self {
        self.messages.push(MessageTemplate::User(
            StringPromptTemplate::from_template(template)
        ));
        self
    }
    
    /// 添加助手消息模板
    pub fn assistant(mut self, template: &str) -> Self {
        self.messages.push(MessageTemplate::Assistant(
            StringPromptTemplate::from_template(template)
        ));
        self
    }
    
    /// 添加自定义角色消息模板
    pub fn custom_role(mut self, role: &str, template: &str) -> Self {
        self.messages.push(MessageTemplate::Custom(
            role.to_string(),
            StringPromptTemplate::from_template(template)
        ));
        self
    }
    
    /// 构建 ChatPromptTemplate
    pub fn build(self) -> ChatPromptTemplate {
        ChatPromptTemplate::new(self.messages)
    }
}

impl ChatPromptTemplate {
    pub fn new(messages: Vec<MessageTemplate>) -> Self {
        let input_variables = Self::extract_input_variables(&messages);
        Self {
            messages,
            input_variables,
            optional_variables: Vec::new(),
            partial_variables: HashMap::new(),
        }
    }
    
    /// 创建构建器
    pub fn builder() -> ChatPromptTemplateBuilder {
        ChatPromptTemplateBuilder::new()
    }
    
    pub fn from_role_templates(role_templates: Vec<(String, String)>) -> Result<Self, TemplateError> {
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
                MessageTemplate::System(template) => variables.extend(template.get_input_variables()),
                MessageTemplate::User(template) => variables.extend(template.get_input_variables()),
                MessageTemplate::Assistant(template) => variables.extend(template.get_input_variables()),
                MessageTemplate::Custom(_, template) => variables.extend(template.get_input_variables()),
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
        
        // 合并部分变量
        let mut merged = self.partial_variables.clone();
        merged.extend(input.clone());
        
        let mut messages = Vec::new();
        
        for message_template in &self.messages {
            let message = match message_template {
                MessageTemplate::System(template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => return Err(TemplateError::FormatError("Expected string from system template".to_string())),
                    };
                    Message::new(Role::System, Content::Text(text))
                }
                MessageTemplate::User(template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => return Err(TemplateError::FormatError("Expected string from user template".to_string())),
                    };
                    Message::new(Role::User, Content::Text(text))
                }
                MessageTemplate::Assistant(template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => return Err(TemplateError::FormatError("Expected string from assistant template".to_string())),
                    };
                    Message::new(Role::Assistant, Content::Text(text))
                }
                MessageTemplate::Custom(role, template) => {
                    let prompt_value = template.format(&merged)?;
                    let text = match prompt_value {
                        PromptValue::String(s) => s,
                        PromptValue::Messages(_) => return Err(TemplateError::FormatError("Expected string from custom template".to_string())),
                    };
                    let role_enum = match role.as_str() {
                        "system" => Role::System,
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        _ => Role::User,
                    };
                    Message::new(role_enum, Content::Text(text))
                }
            };
            messages.push(message);
        }
        
        Ok(PromptValue::Messages(messages))
    }
    
    fn format_async(&self, input: &HashMap<String, String>) -> Pin<Box<dyn Future<Output = Result<Self::Output, TemplateError>> + Send + '_>> {
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
        // 检查必需变量
        let missing: Vec<_> = self.input_variables.iter()
            .filter(|var| !input.contains_key(*var) && !self.partial_variables.contains_key(*var))
            .collect();
        
        if !missing.is_empty() {
            return Err(TemplateError::MissingVariable(
                format!("Missing required variables: {:?}", missing)
            ));
        }
        
        Ok(())
    }
    
    fn template_type(&self) -> &'static str {
        "chat"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_chat_template_builder() {
        let template = ChatPromptTemplate::builder()
            .system("You are a helpful assistant. User's name is {name}.")
            .user("Hello, my name is {name}.")
            .assistant("Hello {name}! How can I help you today?")
            .build();

        let mut input = HashMap::new();
        input.insert("name".to_string(), "Alice".to_string());

        let result = template.format(&input).unwrap();
        match result {
            PromptValue::Messages(messages) => {
                assert_eq!(messages.len(), 3);
                assert_eq!(messages[0].role.as_str(), "system");
                assert!(messages[0].content.as_text().unwrap().contains("Alice"));
            }
            _ => panic!("Expected Messages variant"),
        }
    }

    #[test]
    fn test_chat_template_from_role_templates() {
        let role_templates = vec![
            ("system".to_string(), "You are {role}.".to_string()),
            ("user".to_string(), "I need help with {task}.".to_string()),
        ];

        let template = ChatPromptTemplate::from_role_templates(role_templates).unwrap();
        
        let mut input = HashMap::new();
        input.insert("role".to_string(), "assistant".to_string());
        input.insert("task".to_string(), "coding".to_string());

        let result = template.format(&input).unwrap();
        match result {
            PromptValue::Messages(messages) => {
                assert_eq!(messages.len(), 2);
                assert!(messages[0].content.as_text().unwrap().contains("assistant"));
                assert!(messages[1].content.as_text().unwrap().contains("coding"));
            }
            _ => panic!("Expected Messages variant"),
        }
    }
}