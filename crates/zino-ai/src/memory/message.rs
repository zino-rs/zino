//! Timestamped message wrapper for memory system
//!
//! This module provides a wrapper for messages that includes timestamp information
//! for tracking when messages were added to memory.

use crate::completions::messages::Message;

/// Timestamped message wrapper used internally by the memory system
#[derive(Debug, Clone)]
pub struct TimestampedMessage {
    /// The actual message content
    pub message: Message,
    /// Unix timestamp when the message was added
    pub timestamp: u64,
}

impl TimestampedMessage {
    /// Create a new timestamped message with current timestamp
    pub fn new(message: Message) -> Self {
        Self {
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
