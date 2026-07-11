use crate::app::TurnOutcome;
use crate::planner::PlannerDecision;
use crate::session::{ChatMessage, Session};
use crate::tools::{ToolConfirmation, ToolOutcome, ToolRegistry};
use serde_json::Value;

pub struct Harness {
    session: Session,
    tools: ToolRegistry,
    pending_tool: Option<PendingToolAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingToolAction {
    tool_name: String,
    arguments: Value,
    prompt: String,
}

impl Harness {
    pub fn new() -> Self {
        Self::with_tool_registry(ToolRegistry::default())
    }

    pub fn with_tool_registry(tools: ToolRegistry) -> Self {
        Self {
            session: Session::new(),
            tools,
            pending_tool: None,
        }
    }

    pub fn clear_session(&mut self) {
        self.session.clear();
        self.pending_tool = None;
    }

    pub fn history(&self) -> &[ChatMessage] {
        self.session.history()
    }

    pub fn handle_pending_input(&mut self, user_text: &str) -> Option<Result<TurnOutcome, String>> {
        let pending = self.pending_tool.as_ref()?;
        let trimmed = user_text.trim();

        if is_confirmation(trimmed) {
            let pending = self.pending_tool.take().expect("pending tool should exist");
            let reply = match self.tools.confirm(&pending.tool_name, pending.arguments) {
                Ok(result) => result.summary,
                Err(err) => format!("执行失败：{err}"),
            };
            self.session.push_turn(user_text, &reply);
            return Some(Ok(TurnOutcome::Reply(format!("助手> {reply}"))));
        }

        if is_cancellation(trimmed) {
            self.pending_tool = None;
            let reply = "已取消。".to_string();
            self.session.push_turn(user_text, &reply);
            return Some(Ok(TurnOutcome::Reply(format!("助手> {reply}"))));
        }

        let reply = format!(
            "我还在等你确认：{} 回复“确认”执行，或回复“取消”。",
            pending.prompt
        );
        self.session.push_turn(user_text, &reply);
        Some(Ok(TurnOutcome::Reply(format!("助手> {reply}"))))
    }

    pub fn handle_decision(
        &mut self,
        user_text: &str,
        decision: PlannerDecision,
    ) -> Result<TurnOutcome, String> {
        match decision {
            PlannerDecision::Chat { reply } => {
                self.session.push_turn(user_text, &reply);
                Ok(TurnOutcome::Reply(format!("助手> {reply}")))
            }
            PlannerDecision::Clarify { clarify_question } => {
                self.session.push_turn(user_text, &clarify_question);
                Ok(TurnOutcome::Reply(format!("助手> {clarify_question}")))
            }
            PlannerDecision::Tool {
                tool_name,
                arguments,
            } => {
                let reply = match self.tools.invoke(&tool_name, arguments) {
                    Ok(ToolOutcome::Completed(result)) => result.summary,
                    Ok(ToolOutcome::NeedsConfirmation(confirmation)) => {
                        self.pending_tool = Some(PendingToolAction::from(confirmation.clone()));
                        format!("{} 回复“确认”后我再执行。", confirmation.prompt)
                    }
                    Err(err) => format!("我没法执行这个工具请求：{err}"),
                };
                self.session.push_turn(user_text, &reply);
                Ok(TurnOutcome::Reply(format!("助手> {reply}")))
            }
            PlannerDecision::Retrieve { retrieve_query } => Ok(TurnOutcome::Reply(format!(
                "助手> 本地检索还没有接入执行层：{retrieve_query}"
            ))),
        }
    }
}

impl From<ToolConfirmation> for PendingToolAction {
    fn from(confirmation: ToolConfirmation) -> Self {
        Self {
            tool_name: confirmation.tool_name,
            arguments: confirmation.data,
            prompt: confirmation.prompt,
        }
    }
}

fn is_confirmation(input: &str) -> bool {
    matches!(
        normalize_control_reply(input).as_str(),
        "确认" | "确定" | "可以" | "执行" | "打开" | "好" | "好的" | "ok" | "okay" | "yes" | "y"
    )
}

fn is_cancellation(input: &str) -> bool {
    matches!(
        normalize_control_reply(input).as_str(),
        "取消" | "算了" | "不用" | "别" | "不要" | "停止" | "cancel" | "no" | "n"
    )
}

fn normalize_control_reply(input: &str) -> String {
    input
        .trim()
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '?' | '？' | '!' | '！' | '.' | '。' | ',' | '，' | ':' | '：'
                )
        })
        .to_lowercase()
}
