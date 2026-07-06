mod ollama;
mod openai;

use crate::config::AppConfig;
use crate::session::ChatMessage;

use ollama::OllamaProvider;
use openai::OpenAiProvider;

pub trait ChatClient {
    fn provider_name<'a>(&self, config: &'a AppConfig) -> &'a str;

    fn chat(
        &mut self,
        config: &AppConfig,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String>;
}

pub struct ConfiguredChatClient;

impl ChatClient for ConfiguredChatClient {
    fn provider_name<'a>(&self, config: &'a AppConfig) -> &'a str {
        config.provider.as_str()
    }

    fn chat(
        &mut self,
        config: &AppConfig,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String> {
        match config.provider.as_str() {
            "ollama" => OllamaProvider.chat(config, history, user_text),
            "openai" => OpenAiProvider.chat(config, history, user_text),
            other => Err(format!(
                "unknown AURORA_PROVIDER `{other}`; expected `ollama` or `openai`"
            )),
        }
    }
}
