use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aurora::app::{App, TurnOutcome};
use aurora::config::AppConfig;
use aurora::harness::ConfirmationDecision;
use aurora::model::{ChatClient, ConfiguredChatClient};
use aurora::session::ChatMessage;

struct PanicClient;

impl ChatClient for PanicClient {
    fn provider_name<'a>(&self, _config: &'a AppConfig) -> &'a str {
        "ollama"
    }

    fn chat(
        &mut self,
        _config: &AppConfig,
        _history: &[ChatMessage],
        _user_text: &str,
    ) -> Result<String, String> {
        panic!("quit should not invoke the model client");
    }
}

struct JsonClient {
    response: String,
}

struct QueueClient {
    responses: Rc<RefCell<Vec<String>>>,
}

impl ChatClient for JsonClient {
    fn provider_name<'a>(&self, _config: &'a AppConfig) -> &'a str {
        "ollama"
    }

    fn chat(
        &mut self,
        _config: &AppConfig,
        _history: &[ChatMessage],
        _user_text: &str,
    ) -> Result<String, String> {
        Ok(self.response.clone())
    }
}

impl ChatClient for QueueClient {
    fn provider_name<'a>(&self, _config: &'a AppConfig) -> &'a str {
        "ollama"
    }

    fn chat(
        &mut self,
        _config: &AppConfig,
        _history: &[ChatMessage],
        _user_text: &str,
    ) -> Result<String, String> {
        let mut responses = self.responses.borrow_mut();
        if responses.is_empty() {
            panic!("model client should not be called again");
        }
        Ok(responses.remove(0))
    }
}

struct CaptureClient {
    response: String,
    seen_user_text: Rc<RefCell<Option<String>>>,
}

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl ChatClient for CaptureClient {
    fn provider_name<'a>(&self, config: &'a AppConfig) -> &'a str {
        config.provider.as_str()
    }

    fn chat(
        &mut self,
        _config: &AppConfig,
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
        provider: "ollama".to_string(),
        model: "test-model".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        openai_api_key: None,
        openai_base_url: "https://api.openai.com".to_string(),
        openai_model: "gpt-4o-mini".to_string(),
        aurora_home: root.join(".aurorapulse"),
        identity_card_path: root.join(".aurorapulse/identity-card.md"),
        current_focus_path: root.join(".aurorapulse/current-focus.md"),
        preferences_path: root.join(".aurorapulse/preferences.json"),
        privacy_rules_path: root.join(".aurorapulse/privacy-rules.json"),
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    let pid = std::process::id();
    let root = std::env::temp_dir().join(format!("aurora-{label}-{pid}-{counter}-{nanos}"));
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
fn pending_tool_denial_is_handled_without_calling_model_again() {
    let config = test_config();
    let responses = Rc::new(RefCell::new(vec![
        r#"{
            "mode": "tool",
            "tool_name": "local_launch.open_app",
            "arguments": {
                "app_name": "Safari"
            }
        }"#
        .to_string(),
    ]));
    let client = QueueClient {
        responses: Rc::clone(&responses),
    };
    let mut app = App::new(config, client);

    let first = app.handle_text("打开 Safari").expect("turn should succeed");
    assert_eq!(
        first,
        TurnOutcome::Confirmation {
            tool_name: "local_launch.open_app".to_string(),
            prompt: "打开 Safari 是一个本地启动动作，需要你确认。".to_string(),
        }
    );

    let second = app
        .resolve_confirmation(ConfirmationDecision::Deny)
        .expect("denial should succeed");
    assert_eq!(second, TurnOutcome::Reply("助手> 已取消。".to_string()));
    assert!(
        responses.borrow().is_empty(),
        "the only model response should have been consumed by the first turn"
    );
}

#[test]
fn model_command_reports_current_provider_and_model() {
    let mut config = test_config();
    config.provider = "openai".to_string();
    config.openai_model = "gpt-5.4".to_string();
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app.handle_text("/model").expect("turn should succeed");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手>\nProvider: openai\nModel: gpt-5.4".to_string())
    );
}

#[test]
fn model_question_is_answered_locally_without_calling_model() {
    let mut config = test_config();
    config.provider = "openai".to_string();
    config.openai_model = "gpt-5.4".to_string();
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app
        .handle_text("你现在的模型是谁？")
        .expect("turn should succeed");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手> 现在是 openai 的 gpt-5.4。".to_string())
    );
}

#[test]
fn assistant_identity_question_is_answered_locally_without_calling_model() {
    let config = test_config();
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app.handle_text("你是谁啊？").expect("turn should succeed");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手> 我是 AuroraPulse，你的本地优先个人上下文助手。".to_string())
    );
}

#[test]
fn user_name_question_reads_name_from_identity_card() {
    let config = test_config();
    write_file(&config.identity_card_path, "# Identity Card\nName: Irin");
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app.handle_text("我叫啥？").expect("turn should succeed");

    assert_eq!(outcome, TurnOutcome::Reply("助手> 你叫 Irin。".to_string()));
}

#[test]
fn user_name_question_reports_identity_summary_when_name_is_missing() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Identity Card\nI am the owner and builder of AuroraPulse.",
    );
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app.handle_text("我叫啥？").expect("turn should succeed");

    assert_eq!(
        outcome,
        TurnOutcome::Reply(
            "助手> 我还没看到你的名字；Identity Card 目前写的是：I am the owner and builder of AuroraPulse."
                .to_string()
        )
    );
}

#[test]
fn incomplete_pronoun_fragment_is_clarified_locally() {
    let config = test_config();
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app.handle_text("我").expect("turn should succeed");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手> 这句还没说完。你想问“我是谁”还是别的？".to_string())
    );
}

#[test]
fn user_request_is_prefixed_with_identity_context_before_model_call() {
    let mut config = test_config();
    config.provider = "openai".to_string();
    config.openai_model = "gpt-5.4".to_string();
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
    assert!(user_text.contains("## Aurora Runtime"));
    assert!(user_text.contains("Provider: openai"));
    assert!(user_text.contains("Model: gpt-5.4"));
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

#[test]
fn context_preview_can_render_cloud_policy() {
    let mut config = test_config();
    config.provider = "openai".to_string();
    write_file(
        &config.identity_card_path,
        "# Identity Card\nPublic line\nlocal-only: hidden line",
    );
    let client = PanicClient;
    let mut app = App::new(config, client);

    let outcome = app
        .handle_text("/context preview openai")
        .expect("preview should succeed");

    match outcome {
        TurnOutcome::Reply(message) => {
            assert!(message.contains("Provider: openai"));
            assert!(message.contains("Policy: Cloud"));
            assert!(message.contains("Public line"));
            assert!(!message.contains("hidden line"));
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn openai_provider_reports_missing_api_key_before_network_call() {
    let mut config = test_config();
    config.provider = "openai".to_string();
    config.openai_api_key = None;
    write_file(
        &config.identity_card_path,
        "# Identity Card\nI am building AuroraPulse.",
    );

    let mut app = App::new(config, ConfiguredChatClient);
    let err = app
        .handle_text("帮我总结一下今天的计划")
        .expect_err("missing key should fail clearly");

    assert!(err.contains("OPENAI_API_KEY"));
}
