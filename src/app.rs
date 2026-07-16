use crate::config::AppConfig;
use crate::context;
use crate::harness::{ConfirmationDecision, Harness};
use crate::model::ChatClient;
use crate::planner::{PlannerDecision, build_system_prompt};

#[derive(Debug, PartialEq, Eq)]
pub enum TurnOutcome {
    Ignored,
    Exit(String),
    Cleared(String),
    Reply(String),
    Confirmation {
        tool_name: String,
        prompt: String,
        allow_always: bool,
    },
    ModelSelection {
        current_model: String,
        models: Vec<String>,
    },
    ModelChanged {
        model: String,
        message: String,
    },
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
            let models = self.client.list_models(&self.config)?;
            return Ok(TurnOutcome::ModelSelection {
                current_model: self.config.active_model().to_string(),
                models,
            });
        }

        if trimmed == "/resume" {
            return Ok(TurnOutcome::Reply(
                "助手> 暂无可恢复的历史会话。".to_string(),
            ));
        }

        if trimmed == "/tools" {
            return Ok(TurnOutcome::Reply(format!(
                "助手>\n{}",
                self.harness.tool_catalog()
            )));
        }

        if trimmed == "/tools log" {
            return Ok(TurnOutcome::Reply(format!(
                "助手>\n{}",
                self.harness.render_tool_logs()
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

        if trimmed.starts_with('/') {
            return Ok(TurnOutcome::Reply(format!("助手> 未知命令：{trimmed}")));
        }

        let local_context = context::load(&self.config)?;
        let provider = self.client.provider_name(&self.config);
        let model_user_text = context::compose_user_prompt(
            &local_context,
            provider,
            self.config.active_model(),
            trimmed,
        );
        let system_prompt = build_system_prompt(&self.harness.tool_catalog());
        let planner_json = self.client.chat(
            &self.config,
            &system_prompt,
            self.harness.history(),
            &model_user_text,
        )?;
        let decision = PlannerDecision::parse(&planner_json)?;

        self.harness.handle_decision(trimmed, decision)
    }

    pub fn resolve_confirmation(
        &mut self,
        decision: ConfirmationDecision,
    ) -> Result<TurnOutcome, String> {
        self.harness.resolve_confirmation(decision)
    }

    pub fn select_model(&mut self, model: &str) -> Result<TurnOutcome, String> {
        let selected = model.trim();
        if selected.is_empty() {
            return Err("模型 ID 不能为空".to_string());
        }

        match self.config.provider.as_str() {
            "openai" => self.config.openai_model = selected.to_string(),
            "ollama" => self.config.model = selected.to_string(),
            other => return Err(format!("未知 model provider：{other}")),
        }

        Ok(TurnOutcome::ModelChanged {
            model: selected.to_string(),
            message: format!("助手> 已切换到 {} 的 {selected}。", self.config.provider),
        })
    }
}

pub fn should_show_thinking_indicator(input: &str) -> bool {
    let trimmed = input.trim();
    !trimmed.is_empty() && !matches!(trimmed, "quit" | "exit") && !trimmed.starts_with('/')
}
