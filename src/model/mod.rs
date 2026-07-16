mod ollama;
mod openai;

use crate::config::AppConfig;
use crate::session::ChatMessage;

use ollama::OllamaProvider;
use openai::OpenAiProvider;

pub trait ChatClient {
    fn provider_name<'a>(&self, config: &'a AppConfig) -> &'a str;

    fn list_models(&self, _config: &AppConfig) -> Result<Vec<String>, String> {
        Err("当前 model provider 不支持读取模型目录".to_string())
    }

    fn chat(
        &mut self,
        config: &AppConfig,
        system_prompt: &str,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String>;
}

pub struct ConfiguredChatClient;

impl ChatClient for ConfiguredChatClient {
    fn provider_name<'a>(&self, config: &'a AppConfig) -> &'a str {
        config.provider.as_str()
    }

    fn list_models(&self, config: &AppConfig) -> Result<Vec<String>, String> {
        match config.provider.as_str() {
            "openai" => OpenAiProvider.list_models(config),
            "ollama" => OllamaProvider.list_models(config),
            other => Err(format!(
                "unknown AURORA_PROVIDER `{other}`; expected `ollama` or `openai`"
            )),
        }
    }

    fn chat(
        &mut self,
        config: &AppConfig,
        system_prompt: &str,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String> {
        match config.provider.as_str() {
            "ollama" => OllamaProvider.chat(config, system_prompt, history, user_text),
            "openai" => OpenAiProvider.chat(config, system_prompt, history, user_text),
            other => Err(format!(
                "unknown AURORA_PROVIDER `{other}`; expected `ollama` or `openai`"
            )),
        }
    }
}
