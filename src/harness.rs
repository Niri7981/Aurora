use crate::app::TurnOutcome;
use crate::planner::PlannerDecision;
use crate::session::{ChatMessage, Session};

pub struct Harness {
    session: Session,
}

impl Harness {
    pub fn new() -> Self {
        Self {
            session: Session::new(),
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
            PlannerDecision::Tool { tool_name, .. } => Ok(TurnOutcome::Reply(format!(
                "助手> 工具 {tool_name} 还没有接入执行层。"
            ))),
            PlannerDecision::Retrieve { retrieve_query } => Ok(TurnOutcome::Reply(format!(
                "助手> 本地检索还没有接入执行层：{retrieve_query}"
            ))),
        }
    }
}
