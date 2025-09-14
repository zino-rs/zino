pub mod buffer;
pub mod config;
pub mod error;
pub mod manager;
pub mod message;
pub mod token_buffer;
pub mod tokenizer;
pub mod window;

// 重新导出主要类型
pub use crate::completions::messages::{Content, Message, Role};
pub use buffer::BufferMemory;
pub use config::{MemoryConfig, MemoryType};
pub use error::{MemoryError, MemoryResult};
pub use manager::MemoryManager;
pub use message::TimestampedMessage;
pub use token_buffer::TokenBufferMemory;
pub use tokenizer::{Tokenizer, SimpleTokenizer, CharTokenizer, FixedTokenizer};
pub use window::WindowMemory;

/// 记忆系统的基础 trait
pub trait Memory: Send + Sync {
    /// 添加消息到记忆
    fn add_message(&self, message: Message) -> MemoryResult<()>;
    
    /// 获取所有消息
    fn get_messages(&self) -> MemoryResult<Vec<Message>>;
    
    /// 清空记忆
    fn clear(&self) -> MemoryResult<()>;
    
    /// 获取记忆大小
    fn size(&self) -> MemoryResult<usize>;
    
    /// 获取最后 N 条消息
    fn get_last_messages(&self, n: usize) -> MemoryResult<Vec<Message>>;
    
    /// 获取记忆类型名称
    fn memory_type(&self) -> &'static str;
    
    /// 获取消息迭代器，避免克隆
    fn iter_messages(&self) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>>;
    
    /// 获取最后N条消息的迭代器
    fn iter_last_messages(&self, n: usize) -> MemoryResult<Box<dyn Iterator<Item = Message> + '_>>;
    
    /// 检查记忆是否为空
    fn is_empty(&self) -> MemoryResult<bool> {
        Ok(self.size()? == 0)
    }
    
    /// 获取记忆统计信息
    fn get_stats(&self) -> MemoryResult<MemoryStats> {
        let messages = self.get_messages()?;
        let mut user_count = 0;
        let mut assistant_count = 0;
        let mut system_count = 0;

        for message in &messages {
            match message.role {
                Role::User => user_count += 1,
                Role::Assistant => assistant_count += 1,
                Role::System => system_count += 1,
            }
        }

        Ok(MemoryStats {
            total_messages: messages.len(),
            user_messages: user_count,
            assistant_messages: assistant_count,
            system_messages: system_count,
            memory_type: self.memory_type().to_string(),
        })
    }
}

/// 记忆统计信息
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub system_messages: usize,
    pub memory_type: String,
}

impl MemoryStats {
    /// 获取对话轮数（用户消息数）
    pub fn get_conversation_turns(&self) -> usize {
        self.user_messages
    }

    /// 获取消息分布比例
    pub fn get_message_distribution(&self) -> (f64, f64, f64) {
        if self.total_messages == 0 {
            return (0.0, 0.0, 0.0);
        }

        let user_ratio = self.user_messages as f64 / self.total_messages as f64;
        let assistant_ratio = self.assistant_messages as f64 / self.total_messages as f64;
        let system_ratio = self.system_messages as f64 / self.total_messages as f64;

        (user_ratio, assistant_ratio, system_ratio)
    }
}

