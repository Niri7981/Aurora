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
            return Ok(TurnOutcome::Reply(self.render_model_status()));
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

        if is_incomplete_fragment(trimmed) {
            return Ok(TurnOutcome::Reply(
                "助手> 这句还没说完。你想问“我是谁”还是别的？".to_string(),
            ));
        }

        if is_model_question(trimmed) {
            return Ok(TurnOutcome::Reply(format!(
                "助手> 现在是 {} 的 {}。",
                self.config.provider,
                self.config.active_model()
            )));
        }

        if is_assistant_identity_question(trimmed) {
            return Ok(TurnOutcome::Reply(
                "助手> 我是 AuroraPulse，你的本地优先个人上下文助手。".to_string(),
            ));
        }

        if is_user_identity_question(trimmed) {
            let local_context = context::load(&self.config)?;
            return Ok(TurnOutcome::Reply(render_user_identity(&local_context)));
        }

        if let Some(outcome) = self.harness.handle_pending_input(trimmed) {
            return outcome;
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

    fn render_model_status(&self) -> String {
        format!(
            "助手>\nProvider: {}\nModel: {}",
            self.config.provider,
            self.config.active_model()
        )
    }
}

pub fn should_show_thinking_indicator(input: &str) -> bool {
    let trimmed = input.trim();
    !trimmed.is_empty()
        && !matches!(trimmed, "quit" | "exit")
        && !trimmed.starts_with('/')
        && !is_incomplete_fragment(trimmed)
        && !is_pending_control_reply(trimmed)
        && !is_model_question(trimmed)
        && !is_assistant_identity_question(trimmed)
        && !is_user_identity_question(trimmed)
}

fn render_user_identity(local_context: &context::LocalContext) -> String {
    if let Some(name) = context::identity_name(local_context) {
        return format!("助手> 你叫 {name}。");
    }

    if let Some(summary) = context::identity_summary(local_context) {
        return format!("助手> 我还没看到你的名字；Identity Card 目前写的是：{summary}");
    }

    "助手> 我还不知道你是谁。先运行 /context init，然后在 identity-card.md 里写你的名字和身份。"
        .to_string()
}

fn is_model_question(input: &str) -> bool {
    let normalized = normalize_question(input);
    normalized.contains("模型")
        && (normalized.contains("谁")
            || normalized.contains("哪个")
            || normalized.contains("当前")
            || normalized.contains("现在")
            || normalized.contains("用"))
}

fn is_assistant_identity_question(input: &str) -> bool {
    matches!(
        normalize_question(input).as_str(),
        "你是谁" | "你是谁啊" | "你叫什么" | "你叫啥"
    )
}

fn is_user_identity_question(input: &str) -> bool {
    matches!(
        normalize_question(input).as_str(),
        "我是谁" | "我是谁啊" | "我叫什么" | "我叫啥"
    )
}

fn is_incomplete_fragment(input: &str) -> bool {
    matches!(normalize_question(input).as_str(), "我" | "你")
}

fn is_pending_control_reply(input: &str) -> bool {
    matches!(
        normalize_question(input).to_lowercase().as_str(),
        "确认"
            | "确定"
            | "可以"
            | "执行"
            | "打开"
            | "好"
            | "好的"
            | "ok"
            | "okay"
            | "yes"
            | "y"
            | "取消"
            | "算了"
            | "不用"
            | "别"
            | "不要"
            | "停止"
            | "cancel"
            | "no"
            | "n"
    )
}

fn normalize_question(input: &str) -> String {
    input
        .trim()
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '?' | '？' | '!' | '！' | '.' | '。' | ',' | '，' | ':' | '：'
                )
        })
        .split_whitespace()
        .collect::<String>()
}
