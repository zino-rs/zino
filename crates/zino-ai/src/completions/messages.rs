use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Content {
    Text(String),
    Multimodal(Vec<serde_json::Value>),
}

impl Content {
    /// 获取文本内容，如果是 Text 类型则返回 Some，否则返回 None
    pub fn as_text(&self) -> Option<&String> {
        match self {
            Content::Text(text) => Some(text),
            Content::Multimodal(_) => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AssistantContent {
    Text(String),
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
}

impl AssistantContent {
    pub fn text(text: &str) -> Self {
        Self::Text(text.to_string())
    }

    pub fn tool_call(id: &str, name: &str, arguments: serde_json::Value) -> Self {
        Self::ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            arguments,
        }
    }
}

impl Message {
    /// 创建新的消息实例
    pub fn new(role: Role, content: Content) -> Self {
        Self { role, content }
    }

    /// 创建系统消息
    pub fn system(text: String) -> Self {
        Self::new(Role::System, Content::Text(text))
    }

    /// 创建用户消息
    pub fn user(text: String) -> Self {
        Self::new(Role::User, Content::Text(text))
    }

    /// 创建助手消息
    pub fn assistant(text: String) -> Self {
        Self::new(Role::Assistant, Content::Text(text))
    }
}
