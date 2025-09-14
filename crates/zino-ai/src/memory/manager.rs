//! Memory manager implementation
//!
//! This module provides a unified memory manager that wraps different memory implementations
//! and provides a consistent interface for memory operations.

use super::{BufferMemory, TokenBufferMemory, WindowMemory};
use super::{Memory, MemoryConfig, MemoryResult, MemoryType};
use crate::completions::messages::{Content, Message, Role};
use std::sync::Arc;

/// Memory manager - unified interface
///
/// This struct provides a unified interface for different memory implementations,
/// allowing easy switching between different memory types.
pub struct MemoryManager {
    memory: Arc<dyn Memory>,
}

impl std::fmt::Debug for MemoryManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryManager")
            .field("memory_type", &self.memory.memory_type())
            .field("size", &self.memory.size())
            .finish()
    }
}

impl MemoryManager {
    /// Create a manager with the specified memory instance
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }

    /// Create a manager with configuration
    pub fn with_config(config: MemoryConfig) -> MemoryResult<Self> {
        config.validate()?;

        let memory: Arc<dyn Memory> = match (config.max_tokens, config.max_messages) {
            (Some(max_tokens), _) => Arc::new(TokenBufferMemory::with_tokenizer(
                max_tokens,
                config.tokenizer,
            )),
            (_, Some(max_messages)) => Arc::new(WindowMemory::new(max_messages)),
            (None, None) => Arc::new(BufferMemory::new()),
        };

        Ok(Self { memory })
    }

    /// Create a buffer memory manager
    pub fn with_buffer_memory() -> Self {
        Self {
            memory: Arc::new(BufferMemory::new()),
        }
    }

    /// Create a sliding window memory manager
    pub fn with_window_memory(max_size: usize) -> Self {
        Self {
            memory: Arc::new(WindowMemory::new(max_size)),
        }
    }

    /// Create a token buffer memory manager
    pub fn with_token_buffer_memory(max_tokens: usize) -> Self {
        Self {
            memory: Arc::new(TokenBufferMemory::new(max_tokens)),
        }
    }

    /// Add a message to memory
    pub fn add_message(&self, message: Message) -> MemoryResult<()> {
        self.memory.add_message(message)
    }

    /// Add a conversation pair (user input and assistant output)
    pub fn add_conversation(
        &self,
        user_input: String,
        assistant_output: String,
    ) -> MemoryResult<()> {
        self.add_message(Message {
            role: Role::User,
            content: Content::Text(user_input),
        })?;
        self.add_message(Message {
            role: Role::Assistant,
            content: Content::Text(assistant_output),
        })?;
        Ok(())
    }

    /// Get all messages from memory
    pub fn get_messages(&self) -> MemoryResult<Vec<Message>> {
        self.memory.get_messages()
    }

    /// Clear all messages from memory
    pub fn clear(&self) -> MemoryResult<()> {
        self.memory.clear()
    }

    /// Get the number of messages in memory
    pub fn size(&self) -> MemoryResult<usize> {
        self.memory.size()
    }

    /// Get the last N messages from memory
    pub fn get_last_messages(&self, n: usize) -> MemoryResult<Vec<Message>> {
        self.memory.get_last_messages(n)
    }

    /// Get memory type information
    pub fn get_memory_type(&self) -> String {
        self.memory.memory_type().to_string()
    }

    /// Check if memory is empty
    pub fn is_empty(&self) -> MemoryResult<bool> {
        self.memory.is_empty()
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryResult<super::MemoryStats> {
        self.memory.get_stats()
    }

    /// Export memory to JSON string
    pub fn export_to_json(&self) -> MemoryResult<String> {
        let messages = self.get_messages()?;
        serde_json::to_string_pretty(&messages)
            .map_err(|e| super::MemoryError::Serialization(e.to_string()))
    }

    /// Import memory from JSON string
    pub fn import_from_json(&self, json: &str) -> MemoryResult<()> {
        let messages: Vec<Message> = serde_json::from_str(json)
            .map_err(|e| super::MemoryError::Deserialization(e.to_string()))?;

        // Clear existing memory
        self.clear()?;

        // Add imported messages
        for message in messages {
            self.add_message(message)?;
        }

        Ok(())
    }

    /// Get formatted conversation history
    pub fn get_formatted_history(&self) -> MemoryResult<String> {
        let messages = self.get_messages()?;
        let mut formatted = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    formatted.push_str(&format!(
                        "System: {}\n",
                        message.content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Role::User => {
                    formatted.push_str(&format!(
                        "User: {}\n",
                        message.content.as_text().unwrap_or(&"".to_string())
                    ));
                }
                Role::Assistant => {
                    formatted.push_str(&format!(
                        "Assistant: {}\n",
                        message.content.as_text().unwrap_or(&"".to_string())
                    ));
                }
            }
        }

        Ok(formatted)
    }
}

pub struct ChatBot {
    memory: MemoryManager,
    name: String,
}

impl ChatBot {
    pub fn new(name: String, memory_type: MemoryType) -> Self {
        let memory = match memory_type {
            MemoryType::Buffer => MemoryManager::with_buffer_memory(),
            MemoryType::Window(size) => MemoryManager::with_window_memory(size),
            MemoryType::TokenBuffer(tokens) => MemoryManager::with_token_buffer_memory(tokens),
        };

        Self { memory, name }
    }

    /// Create a chatbot with buffer memory
    pub fn with_buffer_memory(name: String) -> Self {
        Self::new(name, MemoryType::Buffer)
    }

    /// Create a chatbot with window memory
    pub fn with_window_memory(name: String, max_size: usize) -> Self {
        Self::new(name, MemoryType::Window(max_size))
    }

    /// Create a chatbot with token buffer memory
    pub fn with_token_buffer_memory(name: String, max_tokens: usize) -> Self {
        Self::new(name, MemoryType::TokenBuffer(max_tokens))
    }

    pub fn chat(&self, user_input: &str) -> Result<String, super::MemoryError> {
        // 简单的模拟回复逻辑
        let response = match user_input.to_lowercase().as_str() {
            input if input.contains("你好") || input.contains("hello") => {
                format!("你好！我是{}，很高兴为您服务！", self.name)
            }
            input if input.contains("帮助") || input.contains("help") => {
                format!(
                    "我是{}，我可以帮助您处理各种问题。请告诉我您需要什么帮助？",
                    self.name
                )
            }
            input if input.contains("时间") || input.contains("time") => {
                format!(
                    "当前时间：{}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                )
            }
            input if input.contains("天气") || input.contains("weather") => {
                "抱歉，我暂时无法获取天气信息。建议您查看天气应用或网站。".to_string()
            }
            _ => {
                format!(
                    "我收到了您的消息：\"{}\"。作为{}，我会尽力帮助您！",
                    user_input, self.name
                )
            }
        };

        let full_response = format!("{}: {}", self.name, response);

        // 保存对话到记忆
        self.memory
            .add_conversation(user_input.to_string(), full_response.clone())?;

        Ok(full_response)
    }

    pub fn get_conversation_history(&self) -> MemoryResult<Vec<Message>> {
        self.memory.get_messages()
    }

    pub fn clear_memory(&self) -> MemoryResult<()> {
        self.memory.clear()
    }

    pub fn get_memory_stats(&self) -> MemoryResult<super::MemoryStats> {
        self.memory.get_stats()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatbot_basic() {
        // 创建聊天机器人
        let bot = ChatBot::with_buffer_memory("测试助手".to_string());

        // 测试基本对话
        let response1 = bot.chat("你好").unwrap();
        assert!(response1.contains("你好！我是测试助手"));

        let response2 = bot.chat("帮助").unwrap();
        assert!(response2.contains("我可以帮助您处理各种问题"));

        let response3 = bot.chat("时间").unwrap();
        assert!(response3.contains("当前时间："));

        // 验证记忆功能
        let history = bot.get_conversation_history().unwrap();
        assert_eq!(history.len(), 6); // 3个对话，每个包含用户和助手消息

        // 验证记忆统计
        let stats = bot.get_memory_stats().unwrap();
        assert_eq!(stats.total_messages, 6);
        assert_eq!(stats.user_messages, 3);
        assert_eq!(stats.assistant_messages, 3);
    }

    #[test]
    fn test_chatbot_memory_types() {
        // 测试不同内存类型
        let bot1 = ChatBot::with_buffer_memory("BufferBot".to_string());
        let bot2 = ChatBot::with_window_memory("WindowBot".to_string(), 10);
        let bot3 = ChatBot::with_token_buffer_memory("TokenBot".to_string(), 1000);

        // 所有类型都应该能正常工作
        let response1 = bot1.chat("测试").unwrap();
        let response2 = bot2.chat("测试").unwrap();
        let response3 = bot3.chat("测试").unwrap();

        assert!(response1.contains("BufferBot"));
        assert!(response2.contains("WindowBot"));
        assert!(response3.contains("TokenBot"));
    }
}
