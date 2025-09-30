//! Tokenizer implementations for memory system
//!
//! This module provides various tokenizer implementations for counting and processing
//! tokens in text messages for memory management.

use crate::memory::error::MemoryResult;

/// Tokenizer trait for counting and processing tokens
pub trait Tokenizer: Send + Sync + std::fmt::Debug {
    /// Count the number of tokens in text
    fn count_tokens(&self, text: &str) -> MemoryResult<usize>;

    /// Encode text to token IDs
    fn encode(&self, text: &str) -> MemoryResult<Vec<u32>>;

    /// Decode token IDs to text
    fn decode(&self, tokens: &[u32]) -> MemoryResult<String>;

    /// Get tokenizer name
    fn name(&self) -> &'static str;
}

/// Simple whitespace-based tokenizer (for testing and simple scenarios)
#[derive(Debug, Clone)]
pub struct SimpleTokenizer {
    /// Whether to consider punctuation as separate tokens
    consider_punctuation: bool,
}

impl SimpleTokenizer {
    /// Create a new simple tokenizer
    pub fn new() -> Self {
        Self {
            consider_punctuation: false,
        }
    }

    /// Create a simple tokenizer that considers punctuation
    pub fn with_punctuation() -> Self {
        Self {
            consider_punctuation: true,
        }
    }

    /// Simple token counting, considering punctuation
    fn count_tokens_simple(&self, text: &str) -> usize {
        if self.consider_punctuation {
            // Split by whitespace and punctuation
            text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
                .filter(|s| !s.is_empty())
                .count()
        } else {
            // Split by whitespace only
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
        // Simple hash encoding, should use real tokenizer in production
        let tokens = text
            .chars()
            .enumerate()
            .map(|(i, c)| (i as u32).wrapping_add(c as u32))
            .collect();
        Ok(tokens)
    }

    fn decode(&self, tokens: &[u32]) -> MemoryResult<String> {
        // Simple decoding, should use real tokenizer in production
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

/// Character-based tokenizer (more accurate token counting)
#[derive(Debug, Clone)]
pub struct CharTokenizer {
    /// Token weight per character
    char_weight: f64,
}

impl CharTokenizer {
    /// Create a new character tokenizer with default weight
    pub fn new() -> Self {
        Self { char_weight: 0.25 } // Average 0.25 tokens per character
    }

    /// Create a character tokenizer with custom weight
    pub fn with_char_weight(weight: f64) -> Self {
        Self {
            char_weight: weight,
        }
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
        Ok(token_count.max(1)) // At least 1 token
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

/// Fixed tokenizer (for testing purposes)
#[derive(Debug, Clone)]
pub struct FixedTokenizer {
    fixed_count: usize,
}

impl FixedTokenizer {
    /// Create a fixed tokenizer that always returns the same token count
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
