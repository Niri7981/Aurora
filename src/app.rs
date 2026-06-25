use crate::config::AppConfig;
use crate::harness::Harness;
use crate::planner::PlannerDecision;
use crate::session::ChatMessage;

pub trait ChatClient {
    fn chat(
        &mut self,
        ollama_url: &str,
        model: &str,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String>;
}

pub struct OllamaChatClient;

impl ChatClient for OllamaChatClient {
    fn chat(
        &mut self,
        ollama_url: &str,
        model: &str,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String> {
        crate::ollama::chat(ollama_url, model, history, user_text)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TurnOutcome {
    Ignored,
    Exit(String),
    Cleared(String),
    Reply(String),
}

pub struct App<C> {
    config: AppConfig,
    harness: Harness,
    client: C,
}

impl<C: ChatClient> App<C> {
    pub fn new(config: AppConfig, client: C) -> Self {
        Self {
            config,
            harness: Harness::new(),
            client,
        }
    }

    pub fn handle_text(&mut self, input: &str) -> Result<TurnOutcome, String> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(TurnOutcome::Ignored);
        }

        if matches!(trimmed, "quit" | "exit") {
            return Ok(TurnOutcome::Exit("助手> 下次见。".to_string()));
        }

        if trimmed == "/clear" {
            self.harness.clear_session();
            return Ok(TurnOutcome::Cleared("助手> 已清空当前会话。".to_string()));
        }

        let planner_json = self.client.chat(
            &self.config.ollama_url,
            &self.config.model,
            self.harness.history(),
            trimmed,
        )?;
        let decision = PlannerDecision::parse(&planner_json)?;

        self.harness.handle_decision(trimmed, decision)
    }
}
