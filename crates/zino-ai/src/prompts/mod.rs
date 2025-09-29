//! Prompt template system for AI applications.
//!
//! This module provides a comprehensive template system for creating and managing
//! prompts with support for multiple formats, variable interpolation, and different
//! template types including string templates, chat templates, and few-shot templates.

pub mod base;
pub mod chat;
pub mod few_shot;
pub mod format;
pub mod string;

// Re-export main types and traits
pub use base::*;
pub use chat::ChatPromptTemplate;
pub use few_shot::FewShotPromptTemplate;
pub use format::*;
pub use string::StringPromptTemplate;

// Re-export completion types
pub use crate::completions::{Content, Message};
