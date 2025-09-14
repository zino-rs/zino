pub mod base;
pub mod chat;
pub mod few_shot;
pub mod format;
pub mod string;

// 重新导出核心类型和 trait
pub use base::*;
pub use chat::ChatPromptTemplate;
pub use few_shot::FewShotPromptTemplate;
pub use format::*;
pub use string::StringPromptTemplate;

// 重新导出常用的枚举和结构体
pub use crate::completions::{Content, Message, Role};
