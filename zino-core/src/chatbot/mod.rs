//! Unified access to different chatbot services.
//!
//! ## Supported chatbot services
//!
//! | Chatbot service  | Description            | Feature flag           |
//! |------------------|------------------------|------------------------|
//! | `openai`         | OpenAI                 | `chatbot-openai`       |
//!

use crate::{application::StaticRecord, error::Error, extension::TomlTableExt, state::State, Map};
use std::sync::LazyLock;
use toml::Table;

mod client;

/// Supported chatbot services.
#[cfg(feature = "chatbot-openai")]
mod chatbot_openai;

pub use client::Chatbot;

#[cfg(feature = "chatbot-openai")]
use chatbot_openai::OpenAiChatCompletion;

/// Underlying trait of all chatbot services for implementors.
pub trait ChatbotService {
    /// Constructs a new chatbot with the configuration,
    /// returning an error if it fails.
    fn try_new_chatbot(config: &Table) -> Result<Chatbot, Error>;

    /// Returns the model.
    fn model(&self) -> &str;

    /// Attempts to send a message to generate chat completions.
    async fn try_send(&self, message: String, options: Option<Map>) -> Result<Vec<String>, Error>;
}

/// Global access to the shared chatbot services.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalChatbot;

impl GlobalChatbot {
    /// Gets the chatbot for the specific service.
    #[inline]
    pub fn get(name: &str) -> Option<&'static Chatbot> {
        SHARED_CHATBOT_SERVICES.find(name)
    }
}

/// Shared chatbot services.
static SHARED_CHATBOT_SERVICES: LazyLock<StaticRecord<Chatbot>> = LazyLock::new(|| {
    let mut chatbot_services = StaticRecord::new();
    if let Some(chatbots) = State::shared().config().get_array("chatbot") {
        for chatbot in chatbots.iter().filter_map(|v| v.as_table()) {
            let service = chatbot.get_str("service").unwrap_or("unkown");
            let name = chatbot.get_str("name").unwrap_or(service);
            let chatbot_service = Chatbot::try_new(service, chatbot)
                .unwrap_or_else(|err| panic!("fail to connect chatbot `{name}`: {err}"));
            chatbot_services.add(name, chatbot_service);
        }
    }
    chatbot_services
});
