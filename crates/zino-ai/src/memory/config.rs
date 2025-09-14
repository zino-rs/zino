use crate::memory::error::{MemoryError, MemoryResult};
use crate::memory::tokenizer::{Tokenizer, SimpleTokenizer};
use std::time::Duration;

/// 记忆系统配置
#[derive(Debug)]
pub struct MemoryConfig {
    /// 最大token数量（None表示无限制）
    pub max_tokens: Option<usize>,
    /// 最大消息数量（None表示无限制）
    pub max_messages: Option<usize>,
    /// 是否启用自动保存
    pub auto_save: bool,
    /// 自动保存间隔
    pub save_interval: Duration,
    /// Tokenizer实例
    pub tokenizer: Box<dyn Tokenizer>,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用统计
    pub enable_stats: bool,
}

impl MemoryConfig {
    /// 创建默认配置
    pub fn default() -> Self {
        Self {
            max_tokens: None,
            max_messages: None,
            auto_save: false,
            save_interval: Duration::from_secs(300), // 5分钟
            tokenizer: Box::new(SimpleTokenizer::new()),
            enable_compression: false,
            enable_stats: true,
        }
    }
    
    /// 创建缓冲区记忆配置
    pub fn buffer() -> Self {
        Self::default()
    }
    
    /// 创建窗口记忆配置
    pub fn window(max_messages: usize) -> Self {
        Self {
            max_messages: Some(max_messages),
            ..Self::default()
        }
    }
    
    /// 创建token缓冲区记忆配置
    pub fn token_buffer(max_tokens: usize) -> Self {
        Self {
            max_tokens: Some(max_tokens),
            ..Self::default()
        }
    }
    
    /// 设置最大token数量
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// 设置最大消息数量
    pub fn with_max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = Some(max_messages);
        self
    }
    
    /// 设置tokenizer
    pub fn with_tokenizer<T: Tokenizer + 'static>(mut self, tokenizer: T) -> Self {
        self.tokenizer = Box::new(tokenizer);
        self
    }
    
    /// 启用自动保存
    pub fn with_auto_save(mut self, interval: Duration) -> Self {
        self.auto_save = true;
        self.save_interval = interval;
        self
    }
    
    /// 启用压缩
    pub fn with_compression(mut self) -> Self {
        self.enable_compression = true;
        self
    }
    
    /// 禁用统计
    pub fn without_stats(mut self) -> Self {
        self.enable_stats = false;
        self
    }
    
    /// 验证配置
    pub fn validate(&self) -> MemoryResult<()> {
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 {
                return Err(MemoryError::Configuration(
                    "max_tokens cannot be zero".to_string()
                ));
            }
        }
        
        if let Some(max_messages) = self.max_messages {
            if max_messages == 0 {
                return Err(MemoryError::Configuration(
                    "max_messages cannot be zero".to_string()
                ));
            }
        }
        
        if self.auto_save && self.save_interval.is_zero() {
            return Err(MemoryError::Configuration(
                "save_interval cannot be zero when auto_save is enabled".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// 获取配置摘要
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(max_tokens) = self.max_tokens {
            parts.push(format!("max_tokens={}", max_tokens));
        }
        
        if let Some(max_messages) = self.max_messages {
            parts.push(format!("max_messages={}", max_messages));
        }
        
        if self.auto_save {
            parts.push(format!("auto_save={}s", self.save_interval.as_secs()));
        }
        
        parts.push(format!("tokenizer={}", self.tokenizer.name()));
        parts.push(format!("compression={}", self.enable_compression));
        parts.push(format!("stats={}", self.enable_stats));
        
        parts.join(", ")
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self::default()
    }
}

/// 记忆类型枚举（用于向后兼容）
#[derive(Debug, Clone)]
pub enum MemoryType {
    Buffer,
    Window(usize),
    TokenBuffer(usize),
}

impl MemoryType {
    /// 转换为配置
    pub fn to_config(&self) -> MemoryConfig {
        match self {
            MemoryType::Buffer => MemoryConfig::buffer(),
            MemoryType::Window(size) => MemoryConfig::window(*size),
            MemoryType::TokenBuffer(tokens) => MemoryConfig::token_buffer(*tokens),
        }
    }
}

