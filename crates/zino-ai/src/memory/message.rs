use crate::completions::messages::Message;

/// 记忆系统内部使用的带时间戳的消息包装器
#[derive(Debug, Clone)]
pub struct TimestampedMessage {
    pub message: Message,
    pub timestamp: u64,
}

impl TimestampedMessage {
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

