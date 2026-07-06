use crate::config::AppConfig;
use crate::context;
use crate::harness::Harness;
use crate::model::ChatClient;
use crate::planner::PlannerDecision;

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

        if trimmed == "/model" {
            return Ok(TurnOutcome::Reply(format!(
                "助手>\nProvider: {}\nModel: {}",
                self.config.provider,
                self.config.active_model()
            )));
        }

        if trimmed == "/context" || trimmed.starts_with("/context preview") {
            let local_context = context::load(&self.config)?;
            let preview_provider = trimmed
                .strip_prefix("/context preview")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| self.client.provider_name(&self.config));
            return Ok(TurnOutcome::Reply(format!(
                "助手>\n{}",
                local_context.render_preview(preview_provider)
            )));
        }

        if trimmed == "/context init" {
            let report = context::init_files(&self.config)?;
            return Ok(TurnOutcome::Reply(format!("助手>\n{}", report.render())));
        }

        let local_context = context::load(&self.config)?;
        let provider = self.client.provider_name(&self.config);
        let model_user_text = context::compose_user_prompt(
            &local_context,
            provider,
            self.config.active_model(),
            trimmed,
        );
        let planner_json =
            self.client
                .chat(&self.config, self.harness.history(), &model_user_text)?;
        let decision = PlannerDecision::parse(&planner_json)?;

        self.harness.handle_decision(trimmed, decision)
    }
}
