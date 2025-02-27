#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]

use toml::Table;
use zino_core::{
    LazyLock, Map, application::StaticRecord, error::Error, extension::TomlTableExt, state::State,
};

mod client;

/// Supported chatbot services.
mod openai;

pub use client::Chatbot;
use openai::OpenAiChatCompletion;

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
