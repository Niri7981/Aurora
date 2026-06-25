use std::path::PathBuf;

use aurora::app::{App, ChatClient, TurnOutcome};
use aurora::config::AppConfig;
use aurora::session::ChatMessage;

struct PanicClient;

impl ChatClient for PanicClient {
    fn chat(
        &mut self,
        _ollama_url: &str,
        _model: &str,
        _history: &[ChatMessage],
        _user_text: &str,
    ) -> Result<String, String> {
        panic!("quit should not invoke the model client");
    }
}

struct JsonClient {
    response: String,
}

impl ChatClient for JsonClient {
    fn chat(
        &mut self,
        _ollama_url: &str,
        _model: &str,
        _history: &[ChatMessage],
        _user_text: &str,
    ) -> Result<String, String> {
        Ok(self.response.clone())
    }
}

fn test_config() -> AppConfig {
    AppConfig {
        workspace: PathBuf::from("/tmp"),
        model: "test-model".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
    }
}

#[test]
fn quit_input_exits_without_calling_model() {
    let config = test_config();
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app.handle_text("quit").expect("turn should succeed");

    assert_eq!(outcome, TurnOutcome::Exit("助手> 下次见。".to_string()));
}

#[test]
fn model_planner_json_is_routed_through_harness() {
    let config = test_config();
    let client = JsonClient {
        response: r#"{
            "mode": "clarify",
            "clarify_question": "你想打开哪个应用？"
        }"#
        .to_string(),
    };
    let mut app = App::new(config, client);

    let outcome = app.handle_text("打开那个").expect("turn should succeed");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手> 你想打开哪个应用？".to_string())
    );
}
