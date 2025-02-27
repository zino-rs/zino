use self::ChatbotClient::*;
use super::{ChatbotService, OpenAiChatCompletion};
use toml::Table;
use zino_core::{Map, bail, error::Error, extension::TomlTableExt};

/// Client for supported chatbot services.
#[non_exhaustive]
pub(super) enum ChatbotClient {
    /// OpenAI
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
            "openai" => OpenAiChatCompletion::try_new_chatbot(config),
            _ => {
                bail!("chatbot service `{}` is unsupported", service);
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
            OpenAi(chat_completion) => chat_completion.model(),
        }
    }

    async fn try_send(&self, message: String, options: Option<Map>) -> Result<Vec<String>, Error> {
        match &self.client {
            OpenAi(chat_completion) => chat_completion.try_send(message, options).await,
        }
    }
}
