use crate::app::TurnOutcome;
use crate::planner::PlannerDecision;
use crate::session::{ChatMessage, Session};
use crate::tools::{ToolConfirmation, ToolOutcome, ToolRegistry};
use serde_json::Value;
use std::collections::HashSet;

pub struct Harness {
    session: Session,
    tools: ToolRegistry,
    pending_tool: Option<PendingToolAction>,
    always_allowed_tools: HashSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingToolAction {
    user_text: String,
    tool_name: String,
    arguments: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationDecision {
    AllowOnce,
    AlwaysAllow,
    Deny,
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
            always_allowed_tools: HashSet::new(),
        }
    }

    pub fn clear_session(&mut self) {
        self.session.clear();
        self.pending_tool = None;
    }

    pub fn history(&self) -> &[ChatMessage] {
        self.session.history()
    }

    pub fn resolve_confirmation(
        &mut self,
        decision: ConfirmationDecision,
    ) -> Result<TurnOutcome, String> {
        let pending = self
            .pending_tool
            .take()
            .ok_or_else(|| "当前没有等待确认的工具动作".to_string())?;

        if decision == ConfirmationDecision::Deny {
            let reply = "已取消。".to_string();
            self.session.push_turn(&pending.user_text, &reply);
            return Ok(TurnOutcome::Reply(format!("助手> {reply}")));
        }

        if decision == ConfirmationDecision::AlwaysAllow {
            self.always_allowed_tools.insert(pending.tool_name.clone());
        }

        let reply = match self.tools.confirm(&pending.tool_name, pending.arguments) {
            Ok(result) => result.summary,
            Err(err) => format!("执行失败：{err}"),
        };
        self.session.push_turn(&pending.user_text, &reply);
        Ok(TurnOutcome::Reply(format!("助手> {reply}")))
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
            } => match self.tools.invoke(&tool_name, arguments) {
                Ok(ToolOutcome::Completed(result)) => {
                    self.session.push_turn(user_text, &result.summary);
                    Ok(TurnOutcome::Reply(format!("助手> {}", result.summary)))
                }
                Ok(ToolOutcome::NeedsConfirmation(confirmation)) => {
                    if self.always_allowed_tools.contains(&confirmation.tool_name) {
                        let reply = match self
                            .tools
                            .confirm(&confirmation.tool_name, confirmation.data)
                        {
                            Ok(result) => result.summary,
                            Err(err) => format!("执行失败：{err}"),
                        };
                        self.session.push_turn(user_text, &reply);
                        return Ok(TurnOutcome::Reply(format!("助手> {reply}")));
                    }

                    let tool_name = confirmation.tool_name.clone();
                    let prompt = confirmation.prompt.clone();
                    self.pending_tool = Some(PendingToolAction::new(user_text, confirmation));
                    Ok(TurnOutcome::Confirmation { tool_name, prompt })
                }
                Err(err) => {
                    let reply = format!("我没法执行这个工具请求：{err}");
                    self.session.push_turn(user_text, &reply);
                    Ok(TurnOutcome::Reply(format!("助手> {reply}")))
                }
            },
            PlannerDecision::Retrieve { retrieve_query } => Ok(TurnOutcome::Reply(format!(
                "助手> 本地检索还没有接入执行层：{retrieve_query}"
            ))),
        }
    }
}

impl PendingToolAction {
    fn new(user_text: &str, confirmation: ToolConfirmation) -> Self {
        Self {
            user_text: user_text.to_string(),
            tool_name: confirmation.tool_name,
            arguments: confirmation.data,
        }
    }
}
