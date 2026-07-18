use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aurora::config::AppConfig;
use aurora::context::{self, LocalContext};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn test_config() -> AppConfig {
    let root = unique_temp_dir("context-loading");
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
fn loads_identity_focus_preferences_privacy_and_project_context() {
    let config = test_config();
    write_file(&config.identity_card_path, "# Identity\nI am the builder.");
    write_file(&config.current_focus_path, "# Focus\nBuild Phase 1.");
    write_file(&config.preferences_path, r#"{"style":"concise"}"#);
    write_file(
        &config.privacy_rules_path,
        r#"{"blocked_without_confirmation":["secrets"]}"#,
    );
    write_file(
        &config.workspace.join("CONTEXT.md"),
        "# Project\nAuroraPulse",
    );

    let loaded = LocalContext::load(&config).expect("context should load");

    assert!(
        loaded
            .identity_card
            .expect("identity")
            .content
            .contains("builder")
    );
    assert!(
        loaded
            .current_focus
            .expect("focus")
            .content
            .contains("Phase 1")
    );
    assert_eq!(loaded.project_contexts.len(), 1);
    assert!(loaded.missing.is_empty());
}

#[test]
fn cloud_model_context_excludes_local_only_lines() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Identity\nPublic context\nlocal-only: keep this on device\nprivate: hidden",
    );

    let loaded = context::load(&config).expect("context should load");
    let rendered = loaded.render_model_context("openai");

    assert!(rendered.contains("Public context"));
    assert!(!rendered.contains("keep this on device"));
    assert!(!rendered.contains("hidden"));
}

#[test]
fn cloud_model_context_uses_configured_redaction_markers() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Identity\nPublic context\nconfidential: custom hidden value",
    );
    write_file(
        &config.privacy_rules_path,
        r#"{"redaction_markers":["confidential:"]}"#,
    );

    let loaded = context::load(&config).expect("context should load");
    let rendered = loaded.render_model_context("openai");

    assert!(rendered.contains("Public context"));
    assert!(!rendered.contains("custom hidden value"));
}

#[test]
fn init_files_creates_templates_without_overwriting_existing_files() {
    let config = test_config();
    write_file(
        &config.identity_card_path,
        "# Custom Identity\nDo not overwrite.",
    );

    let report = context::init_files(&config).expect("init should succeed");

    assert!(report.existing.contains(&config.identity_card_path));
    assert!(report.created.contains(&config.current_focus_path));
    assert!(report.created.contains(&config.preferences_path));
    assert!(report.created.contains(&config.privacy_rules_path));

    let identity =
        fs::read_to_string(&config.identity_card_path).expect("identity should be readable");
    assert!(identity.contains("Do not overwrite."));
}
