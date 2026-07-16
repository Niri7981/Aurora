use crate::app::TurnOutcome;
use crate::planner::PlannerDecision;
use crate::session::{ChatMessage, Session};
use crate::tools::{ToolConfirmation, ToolOutcome, ToolRegistry, ToolResult, ToolRisk};
use serde_json::Value;
use std::collections::HashSet;
use std::time::Instant;

const MAX_TOOL_LOGS: usize = 32;

pub struct Harness {
    session: Session,
    tools: ToolRegistry,
    pending_tool: Option<PendingToolAction>,
    always_allowed_tools: HashSet<String>,
    tool_logs: Vec<ToolInvocationLog>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingToolAction {
    user_text: String,
    tool_name: String,
    arguments: Value,
    risk: ToolRisk,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolInvocationLog {
    pub result: ToolResult,
    pub elapsed_ms: u128,
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
            tool_logs: Vec::new(),
        }
    }

    pub fn clear_session(&mut self) {
        self.session.clear();
        self.pending_tool = None;
    }

    pub fn history(&self) -> &[ChatMessage] {
        self.session.history()
    }

    pub fn tool_catalog(&self) -> String {
        self.tools.planner_catalog()
    }

    pub fn tool_logs(&self) -> &[ToolInvocationLog] {
        &self.tool_logs
    }

    pub fn render_tool_logs(&self) -> String {
        if self.tool_logs.is_empty() {
            return "暂无工具调用记录。".to_string();
        }

        self.tool_logs
            .iter()
            .rev()
            .map(|entry| {
                format!(
                    "{}  {:<9}  {:>4}ms  {}",
                    entry.result.tool_name,
                    entry.result.status,
                    entry.elapsed_ms,
                    entry.result.summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
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
            let result = ToolResult::denied(pending.tool_name, pending.arguments);
            return Ok(self.finish_tool_result(&pending.user_text, result, 0));
        }

        if decision == ConfirmationDecision::AlwaysAllow && pending.risk.allows_session_bypass() {
            self.always_allowed_tools.insert(pending.tool_name.clone());
        }

        let started = Instant::now();
        let result = self.tools.confirm(&pending.tool_name, pending.arguments);
        Ok(self.finish_tool_result(&pending.user_text, result, started.elapsed().as_millis()))
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
                let started = Instant::now();
                match self.tools.invoke(&tool_name, arguments) {
                    ToolOutcome::Completed(result) => Ok(self.finish_tool_result(
                        user_text,
                        result,
                        started.elapsed().as_millis(),
                    )),
                    ToolOutcome::NeedsConfirmation(confirmation) => {
                        if confirmation.risk.allows_session_bypass()
                            && self.always_allowed_tools.contains(&confirmation.tool_name)
                        {
                            let started = Instant::now();
                            let result = self
                                .tools
                                .confirm(&confirmation.tool_name, confirmation.data);
                            return Ok(self.finish_tool_result(
                                user_text,
                                result,
                                started.elapsed().as_millis(),
                            ));
                        }

                        let tool_name = confirmation.tool_name.clone();
                        let prompt = confirmation.prompt.clone();
                        let allow_always = confirmation.risk.allows_session_bypass();
                        self.pending_tool = Some(PendingToolAction::new(user_text, confirmation));
                        Ok(TurnOutcome::Confirmation {
                            tool_name,
                            prompt,
                            allow_always,
                        })
                    }
                }
            }
            PlannerDecision::Retrieve { retrieve_query } => Ok(TurnOutcome::Reply(format!(
                "助手> 本地检索还没有接入执行层：{retrieve_query}"
            ))),
        }
    }

    fn finish_tool_result(
        &mut self,
        user_text: &str,
        result: ToolResult,
        elapsed_ms: u128,
    ) -> TurnOutcome {
        let reply = result.summary.clone();
        self.session.push_turn(user_text, &reply);
        self.tool_logs
            .push(ToolInvocationLog { result, elapsed_ms });
        if self.tool_logs.len() > MAX_TOOL_LOGS {
            self.tool_logs.remove(0);
        }
        TurnOutcome::Reply(format!("助手> {reply}"))
    }
}

impl PendingToolAction {
    fn new(user_text: &str, confirmation: ToolConfirmation) -> Self {
        Self {
            user_text: user_text.to_string(),
            tool_name: confirmation.tool_name,
            arguments: confirmation.data,
            risk: confirmation.risk,
        }
    }
}
