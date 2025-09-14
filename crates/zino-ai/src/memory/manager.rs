use super::{Memory, MemoryResult, MemoryConfig, MemoryType};
use super::{BufferMemory, TokenBufferMemory, WindowMemory};
use crate::completions::messages::{Content, Message, Role};
use std::sync::Arc;

// MemoryType 已移动到 config.rs

/// 记忆管理器 - 统一接口
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
    /// 使用指定的记忆实例创建管理器
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }

    /// 使用配置创建管理器
    pub fn with_config(config: MemoryConfig) -> MemoryResult<Self> {
        config.validate()?;
        
        let memory: Arc<dyn Memory> = match (config.max_tokens, config.max_messages) {
            (Some(max_tokens), _) => {
                Arc::new(TokenBufferMemory::with_tokenizer(max_tokens, config.tokenizer))
            }
            (_, Some(max_messages)) => {
                Arc::new(WindowMemory::new(max_messages))
            }
            (None, None) => {
                Arc::new(BufferMemory::new())
            }
        };
        
        Ok(Self { memory })
    }

    /// 创建缓冲区记忆管理器
    pub fn with_buffer_memory() -> Self {
        Self {
            memory: Arc::new(BufferMemory::new()),
        }
    }

    /// 创建滑动窗口记忆管理器
    pub fn with_window_memory(max_size: usize) -> Self {
        Self {
            memory: Arc::new(WindowMemory::new(max_size)),
        }
    }

    /// 创建 Token 缓冲区记忆管理器
    pub fn with_token_buffer_memory(max_tokens: usize) -> Self {
        Self {
            memory: Arc::new(TokenBufferMemory::new(max_tokens)),
        }
    }

    /// 添加消息到记忆
    pub fn add_message(&self, message: Message) -> MemoryResult<()> {
        self.memory.add_message(message)
    }

    /// 添加用户和助手的对话对
    pub fn add_conversation(&self, user_input: String, assistant_output: String) -> MemoryResult<()> {
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

    /// 获取所有消息
    pub fn get_messages(&self) -> MemoryResult<Vec<Message>> {
        self.memory.get_messages()
    }

    /// 清空记忆
    pub fn clear(&self) -> MemoryResult<()> {
        self.memory.clear()
    }

    /// 获取记忆大小
    pub fn size(&self) -> MemoryResult<usize> {
        self.memory.size()
    }

    /// 获取最后 N 条消息
    pub fn get_last_messages(&self, n: usize) -> MemoryResult<Vec<Message>> {
        self.memory.get_last_messages(n)
    }

    /// 获取记忆类型信息
    pub fn get_memory_type(&self) -> String {
        self.memory.memory_type().to_string()
    }

    /// 检查记忆是否为空
    pub fn is_empty(&self) -> MemoryResult<bool> {
        self.memory.is_empty()
    }

    /// 获取记忆统计信息
    pub fn get_stats(&self) -> MemoryResult<super::MemoryStats> {
        self.memory.get_stats()
    }

    /// 导出记忆为 JSON 字符串
    pub fn export_to_json(&self) -> MemoryResult<String> {
        let messages = self.get_messages()?;
        serde_json::to_string_pretty(&messages).map_err(|e| {
            super::MemoryError::Serialization(e.to_string())
        })
    }

    /// 从 JSON 字符串导入记忆
    pub fn import_from_json(&self, json: &str) -> MemoryResult<()> {
        let messages: Vec<Message> = serde_json::from_str(json).map_err(|e| {
            super::MemoryError::Deserialization(e.to_string())
        })?;

        // 清空现有记忆
        self.clear()?;

        // 添加导入的消息
        for message in messages {
            self.add_message(message)?;
        }

        Ok(())
    }

    /// 获取格式化的对话历史
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

// MemoryStats 已移动到 mod.rs

/// 聊天机器人示例 - 展示如何在实际项目中使用
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

    pub fn chat(&self, user_input: &str) -> Result<String, super::MemoryError> {
        // 在实际项目中，这里会调用你的 AI 模型
        let response = match user_input {
            "什么是所有权？" => "所有权是 Rust 的核心概念，确保内存安全。",
            "什么是借用？" => "借用允许你使用值而不获取所有权。",
            "什么是生命周期？" => "生命周期确保引用在使用期间有效。",
            _ => "这是一个关于 Rust 的问题，让我为你解答。",
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

