use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aurora::config::AppConfig;
use aurora::mcp::AuroraContextService;
use serde_json::Value;

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn test_config() -> AppConfig {
    let root = unique_temp_dir("mcp-identity");
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
    let root = std::env::temp_dir().join(format!(
        "aurora-{label}-{}-{counter}-{nanos}",
        std::process::id()
    ));
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
fn identity_pack_is_filtered_source_aware_and_audited() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Identity\nName: Irin\nprivate: hidden\nconfidential: also hidden",
    );
    write_file(
        &config.privacy_rules_path,
        r#"{"redaction_markers":["private:","confidential:"]}"#,
    );
    let service = AuroraContextService::new(config.clone(), "codex");

    let pack = service
        .get_identity("understand who the user is")
        .expect("identity pack should load");

    assert_eq!(pack.client, "codex");
    assert_eq!(pack.items.len(), 1);
    assert_eq!(pack.items[0].source, "aurora://identity-card.md");
    assert!(pack.items[0].content.contains("Name: Irin"));
    assert!(!pack.items[0].content.contains("hidden"));
    assert_eq!(pack.omissions[0].line_count, 2);
    assert!(
        !serde_json::to_string(&pack)
            .expect("pack should serialize")
            .contains(&config.aurora_home.display().to_string())
    );

    let audit = fs::read_to_string(service.audit_log_path()).expect("audit log should exist");
    let event: Value = serde_json::from_str(audit.trim()).expect("audit line should be JSON");
    assert_eq!(event["client"], "codex");
    assert_eq!(event["tool"], "get_identity");
    assert_eq!(event["status"], "succeeded");
    assert_eq!(event["omitted_lines"], 2);
    assert_eq!(event["returned_sources"][0], "aurora://identity-card.md");
}

#[test]
fn personal_context_search_is_bounded_and_excludes_privacy_rules() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Identity\nI am building AuroraPulse.",
    );
    write_file(
        &config.current_focus_path,
        "# Focus\nShip the Aurora MCP identity server.",
    );
    write_file(
        &config.preferences_path,
        r#"{"style":"concise","project":"AuroraPulse"}"#,
    );
    write_file(
        &config.privacy_rules_path,
        r#"{"redaction_markers":["private:"]}"#,
    );
    write_file(
        &config.workspace.join("CONTEXT.md"),
        "# Project\nAuroraPulse roadmap and architecture.",
    );
    let service = AuroraContextService::new(config, "codex");

    let pack = service
        .search_personal_context("AuroraPulse 下一步", "plan the user's project", Some(2))
        .expect("search should succeed");

    assert_eq!(pack.items.len(), 2);
    assert!(pack.items.iter().all(|item| item.label != "Privacy Rules"));
    assert!(pack.items.iter().any(|item| item.category == "identity"));
    assert!(
        pack.items
            .iter()
            .any(|item| item.category == "current_focus")
    );
}

#[test]
fn missing_optional_context_returns_an_empty_audited_pack() {
    let config = test_config();
    let service = AuroraContextService::new(config, "codex");

    let pack = service
        .get_current_focus("continue the user's current work")
        .expect("missing optional focus should not fail");

    assert!(pack.items.is_empty());
    let audit = fs::read_to_string(service.audit_log_path()).expect("audit log should exist");
    assert!(audit.contains("\"status\":\"succeeded\""));
}

#[test]
fn empty_search_query_is_rejected_without_reading_context() {
    let config = test_config();
    let service = AuroraContextService::new(config, "codex");

    let error = service
        .search_personal_context("  ", "test", None)
        .expect_err("empty query should fail");

    assert_eq!(error, "query must not be empty");
    assert!(!service.audit_log_path().exists());
}
