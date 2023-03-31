use self::ChatbotClient::*;
use super::ChatbotService;
use crate::{error::Error, extend::TomlTableExt, Map};
use toml::Table;

#[cfg(feature = "chatbot-openai")]
use super::OpenAiChatCompletion;

/// Client for supported chatbot services.
#[non_exhaustive]
pub(super) enum ChatbotClient {
    /// OpenAI
    #[cfg(feature = "chatbot-openai")]
    OpenAi(OpenAiChatCompletion),
}

/// A chatbot with the specific service and model.
pub struct Chatbot {
    /// Service
    service: String,
    /// Name
    name: String,
    /// Client
    client: ChatbotClient,
}

impl Chatbot {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new(
        service: impl Into<String>,
        name: impl Into<String>,
        client: ChatbotClient,
    ) -> Self {
        Self {
            service: service.into(),
            name: name.into(),
            client,
        }
    }

    /// Constructs a new instance with the service and configuration,
    /// returning an error if it fails.
    pub fn try_new(service: &str, config: &Table) -> Result<Chatbot, Error> {
        match service {
            #[cfg(feature = "chatbot-openai")]
            "openai" => OpenAiChatCompletion::try_new_chatbot(config),
            _ => {
                let message = format!("chatbot service `{service}` is unsupported");
                return Err(Error::new(message));
            }
        }
    }

    /// Returns the service.
    #[inline]
    pub fn service(&self) -> &str {
        self.service.as_str()
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ChatbotService for Chatbot {
    fn try_new_chatbot(config: &Table) -> Result<Chatbot, Error> {
        let service = config.get_str("service").unwrap_or("unkown");
        Self::try_new(service, config)
    }

    fn model(&self) -> &str {
        match &self.client {
            #[cfg(feature = "chatbot-openai")]
            OpenAi(chat_completion) => chat_completion.model(),
        }
    }

    async fn try_send(&self, message: String, options: Option<Map>) -> Result<Vec<String>, Error> {
        match &self.client {
            #[cfg(feature = "chatbot-openai")]
            OpenAi(chat_completion) => chat_completion.try_send(message, options).await,
        }
    }
}
