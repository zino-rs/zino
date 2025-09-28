//! Baidu AI model provider implementation.
//!
//! This module provides client and completion functionality for Baidu's
//! AI models, including support for both standard and streaming completions.

/// HTTP client for Baidu AI API.
pub mod client;

/// Completion models and streaming support for Baidu AI.
pub mod completion;
