const MAX_HISTORY_MESSAGES: usize = 16;

#[derive(Clone)]
pub struct ChatMessage {
    pub role: &'static str,
    pub content: String,
}

pub struct Session {
    history: Vec<ChatMessage>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn history(&self) -> &[ChatMessage] {
        &self.history
    }

    pub fn push_turn(&mut self, user_text: &str, assistant_text: &str) {
        self.history.push(ChatMessage {
            role: "user",
            content: user_text.to_string(),
        });
        self.history.push(ChatMessage {
            role: "assistant",
            content: assistant_text.to_string(),
        });
        self.trim_history();
    }

    fn trim_history(&mut self) {
        if self.history.len() > MAX_HISTORY_MESSAGES {
            let excess = self.history.len() - MAX_HISTORY_MESSAGES;
            self.history.drain(0..excess);
        }
    }
}
