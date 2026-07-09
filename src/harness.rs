use crate::app::TurnOutcome;
use crate::planner::PlannerDecision;
use crate::session::{ChatMessage, Session};
use crate::tools::{ToolOutcome, ToolRegistry};

pub struct Harness {
    session: Session,
    tools: ToolRegistry,
}

impl Harness {
    pub fn new() -> Self {
        Self::with_tool_registry(ToolRegistry::default())
    }

    pub fn with_tool_registry(tools: ToolRegistry) -> Self {
        Self {
            session: Session::new(),
            tools,
        }
    }

    pub fn clear_session(&mut self) {
        self.session.clear();
    }

    pub fn history(&self) -> &[ChatMessage] {
        self.session.history()
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
