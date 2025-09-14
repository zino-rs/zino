use crate::memory::error::MemoryResult;

/// Token计算器trait
pub trait Tokenizer: Send + Sync + std::fmt::Debug {
    /// 计算文本的token数量
    fn count_tokens(&self, text: &str) -> MemoryResult<usize>;
    
    /// 将文本编码为token IDs
    fn encode(&self, text: &str) -> MemoryResult<Vec<u32>>;
    
    /// 将token IDs解码为文本
    fn decode(&self, tokens: &[u32]) -> MemoryResult<String>;
    
    /// 获取tokenizer名称
    fn name(&self) -> &'static str;
}

/// 简单的空格分词器（用于测试和简单场景）
#[derive(Debug, Clone)]
pub struct SimpleTokenizer {
    /// 是否考虑标点符号
    consider_punctuation: bool,
}

impl SimpleTokenizer {
    pub fn new() -> Self {
        Self {
            consider_punctuation: false,
        }
    }
    
    pub fn with_punctuation() -> Self {
        Self {
            consider_punctuation: true,
        }
    }
    
    /// 简单的token计数，考虑标点符号
    fn count_tokens_simple(&self, text: &str) -> usize {
        if self.consider_punctuation {
            // 按空格和标点符号分词
            text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
                .filter(|s| !s.is_empty())
                .count()
        } else {
            // 只按空格分词
            text.split_whitespace().count()
        }
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for SimpleTokenizer {
    fn count_tokens(&self, text: &str) -> MemoryResult<usize> {
        Ok(self.count_tokens_simple(text))
    }
    
    fn encode(&self, text: &str) -> MemoryResult<Vec<u32>> {
        // 简单的hash编码，实际应用中应该使用真正的tokenizer
        let tokens = text
            .chars()
            .enumerate()
            .map(|(i, c)| (i as u32).wrapping_add(c as u32))
            .collect();
        Ok(tokens)
    }
    
    fn decode(&self, tokens: &[u32]) -> MemoryResult<String> {
        // 简单的解码，实际应用中应该使用真正的tokenizer
        let text = tokens
            .iter()
            .map(|&token| char::from_u32(token).unwrap_or('?'))
            .collect();
        Ok(text)
    }
    
    fn name(&self) -> &'static str {
        "SimpleTokenizer"
    }
}

/// 基于字符的tokenizer（更准确的token计数）
#[derive(Debug, Clone)]
pub struct CharTokenizer {
    /// 每个字符的token权重
    char_weight: f64,
}

impl CharTokenizer {
    pub fn new() -> Self {
        Self { char_weight: 0.25 } // 平均每个字符0.25个token
    }
    
    pub fn with_char_weight(weight: f64) -> Self {
        Self { char_weight: weight }
    }
}

impl Default for CharTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for CharTokenizer {
    fn count_tokens(&self, text: &str) -> MemoryResult<usize> {
        let char_count = text.chars().count();
        let token_count = (char_count as f64 * self.char_weight).ceil() as usize;
        Ok(token_count.max(1)) // 至少1个token
    }
    
    fn encode(&self, text: &str) -> MemoryResult<Vec<u32>> {
        Ok(text.chars().map(|c| c as u32).collect())
    }
    
    fn decode(&self, tokens: &[u32]) -> MemoryResult<String> {
        let text = tokens
            .iter()
            .filter_map(|&token| char::from_u32(token))
            .collect();
        Ok(text)
    }
    
    fn name(&self) -> &'static str {
        "CharTokenizer"
    }
}

/// 固定tokenizer（用于测试）
#[derive(Debug, Clone)]
pub struct FixedTokenizer {
    fixed_count: usize,
}

impl FixedTokenizer {
    pub fn new(fixed_count: usize) -> Self {
        Self { fixed_count }
    }
}

impl Tokenizer for FixedTokenizer {
    fn count_tokens(&self, _text: &str) -> MemoryResult<usize> {
        Ok(self.fixed_count)
    }
    
    fn encode(&self, text: &str) -> MemoryResult<Vec<u32>> {
        Ok(text.bytes().map(|b| b as u32).collect())
    }
    
    fn decode(&self, tokens: &[u32]) -> MemoryResult<String> {
        let text = tokens
            .iter()
            .filter_map(|&token| u8::try_from(token).ok().map(|b| b as char))
            .collect();
        Ok(text)
    }
    
    fn name(&self) -> &'static str {
        "FixedTokenizer"
    }
}

