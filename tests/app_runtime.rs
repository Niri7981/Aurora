use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

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

struct CaptureClient {
    response: String,
    seen_user_text: Rc<RefCell<Option<String>>>,
}

impl ChatClient for CaptureClient {
    fn chat(
        &mut self,
        _ollama_url: &str,
        _model: &str,
        _history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String> {
        *self.seen_user_text.borrow_mut() = Some(user_text.to_string());
        Ok(self.response.clone())
    }
}

fn test_config() -> AppConfig {
    let root = unique_temp_dir("app-runtime");
    AppConfig {
        workspace: root.clone(),
        model: "test-model".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        aurora_home: root.join(".aurorapulse"),
        identity_card_path: root.join(".aurorapulse/identity-card.md"),
        current_focus_path: root.join(".aurorapulse/current-focus.md"),
        preferences_path: root.join(".aurorapulse/preferences.json"),
        privacy_rules_path: root.join(".aurorapulse/privacy-rules.json"),
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("aurora-{label}-{nanos}"));
    fs::create_dir_all(&root).expect("temp dir should be created");
    root
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir should be created");
    }
    fs::write(path, content).expect("file should be written");
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

#[test]
fn user_request_is_prefixed_with_identity_context_before_model_call() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Identity Card\nI am building AuroraPulse.",
    );
    write_file(
        &config.current_focus_path,
        "# Current Focus\nShip Phase 1 identity context.",
    );

    let seen_user_text = Rc::new(RefCell::new(None));
    let client = CaptureClient {
        response: r#"{"mode":"chat","reply":"收到。"}"#.to_string(),
        seen_user_text: Rc::clone(&seen_user_text),
    };
    let mut app = App::new(config, client);

    let outcome = app
        .handle_text("我下一步应该做什么？")
        .expect("turn should succeed");

    assert_eq!(outcome, TurnOutcome::Reply("助手> 收到。".to_string()));
    let user_text = seen_user_text
        .borrow()
        .clone()
        .expect("model should receive a user message");
    assert!(user_text.contains("## Identity Card"));
    assert!(user_text.contains("I am building AuroraPulse."));
    assert!(user_text.contains("## Current Focus"));
    assert!(user_text.contains("Current user request:"));
    assert!(user_text.contains("我下一步应该做什么？"));
}

#[test]
fn context_init_creates_local_identity_files() {
    let config = test_config();
    let client = PanicClient;
    let mut app = App::new(config.clone(), client);

    let outcome = app
        .handle_text("/context init")
        .expect("context init should succeed");

    match outcome {
        TurnOutcome::Reply(message) => {
            assert!(message.contains("Context files ready."));
            assert!(message.contains("identity-card.md"));
        }
        other => panic!("unexpected outcome: {other:?}"),
    }

    assert!(config.identity_card_path.exists());
    assert!(config.current_focus_path.exists());
    assert!(config.preferences_path.exists());
    assert!(config.privacy_rules_path.exists());
}
