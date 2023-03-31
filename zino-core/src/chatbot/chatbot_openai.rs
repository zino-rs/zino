use super::{chatbot::ChatbotClient::OpenAi, Chatbot, ChatbotService};
use crate::{
    application::http_client,
    error::Error,
    extend::{JsonObjectExt, TomlTableExt},
    Map,
};
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Chat, Client,
};
use futures::StreamExt;
use toml::Table;

/// OpenAI chat completion.
pub(super) struct OpenAiChatCompletion {
    /// Model
    model: String,
    /// Client
    client: Client,
}

impl OpenAiChatCompletion {
    /// Creates a new instance.
    pub(super) fn new(model: impl Into<String>, client: Client) -> Self {
        Self {
            model: model.into(),
            client,
        }
    }

    /// Returns a chat conversation.
    #[inline]
    pub(super) fn chat(&self) -> Chat {
        self.client.chat()
    }
}

impl ChatbotService for OpenAiChatCompletion {
    fn try_new_chatbot(config: &Table) -> Result<Chatbot, Error> {
        let name = config.get_str("name").unwrap_or("openai");
        let model = config.get_str("model").unwrap_or("gpt-3.5-turbo");

        let mut client = Client::new();
        if let Some(reqwest_client) = http_client::SHARED_HTTP_CLIENT.get() {
            client = client.with_http_client(reqwest_client.clone());
        }
        if let Some(api_key) = config.get_str("api-key") {
            client = client.with_api_key(api_key);
        }
        if let Some(org_id) = config.get_str("org-id") {
            client = client.with_org_id(org_id);
        }
        if let Some(api_base) = config.get_str("api-base") {
            client = client.with_api_base(api_base);
        }

        let chat_completion = OpenAiChatCompletion::new(model, client);
        let chatbot = Chatbot::new("openapi", name, OpenAi(chat_completion));
        Ok(chatbot)
    }

    #[inline]
    fn model(&self) -> &str {
        self.model.as_str()
    }

    async fn try_send(&self, message: String, options: Option<Map>) -> Result<Vec<String>, Error> {
        let request_message = ChatCompletionRequestMessageArgs::default()
            .content(message)
            .role(Role::User)
            .build()?;

        let mut sampling_temperature = 0.5;
        let mut num_choices = 1;
        let mut max_tokens = 4096;
        if let Some(options) = options {
            if let Some(temperature) = options.get_f32("temperature") {
                sampling_temperature = temperature;
            }
            if let Some(choices) = options.get_u8("num-choices") {
                num_choices = choices;
            }
            if let Some(tokens) = options.get_u16("max-tokens") {
                max_tokens = tokens;
            }
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model())
            .messages([request_message])
            .temperature(sampling_temperature)
            .n(num_choices)
            .max_tokens(max_tokens)
            .build()?;
        let mut stream = self.chat().create_stream(request).await?;
        let mut data: Vec<String> = Vec::new();
        while let Some(response) = stream.next().await {
            for (index, choice) in response?.choices.iter().enumerate() {
                if let Some(ref content) = choice.delta.content {
                    if let Some(output) = data.get_mut(index) {
                        output.push_str(content);
                    } else {
                        data.push(content.to_owned());
                    }
                }
            }
        }
        Ok(data)
    }
}
