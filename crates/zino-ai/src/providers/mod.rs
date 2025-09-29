//! AI model providers for different vendors.
//!
//! This module provides implementations for various AI model providers,
//! including Baidu, MiniMax, Qwen, and Zhipu. Each provider offers
//! both standard completion and streaming completion capabilities.

/// Baidu AI model provider implementation.
pub mod baidu;

/// MiniMax AI model provider implementation.
pub mod minimax;

/// Zhipu AI model provider implementation.
pub mod zhipu;

/// Qwen AI model provider implementation.
pub mod qwen;
