use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aurora::application::aurora_search_service::{AuroraSearchService, SearchAurora};
use aurora::application::context_gateway::ContextGateway;
use aurora::application::telegram_message_service::{SaveTelegramMessage, TelegramMessageService};
use aurora::config::AppConfig;
use aurora::infrastructure::audit_log::AuditLog;
use sqlx::postgres::PgPoolOptions;

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_dir() -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "aurora-global-search-{}-{counter}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("test directory should be created");
    path
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, content).expect("fixture file should be written");
}

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn globally_searches_personal_context_and_telegram_with_server_sources() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("PostgreSQL should be available");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations should run");

    let root = unique_temp_dir();
    let config = AppConfig {
        workspace: root.clone(),
        aurora_home: root.join(".aurorapulse"),
        identity_card_path: root.join(".aurorapulse/identity-card.md"),
        current_focus_path: root.join(".aurorapulse/current-focus.md"),
        preferences_path: root.join(".aurorapulse/preferences.json"),
        privacy_rules_path: root.join(".aurorapulse/privacy-rules.json"),
    };
    write_file(
        &config.identity_card_path,
        "# Identity\nBuilding Aurora.\nprivate: never disclose this line",
    );
    write_file(
        &config.privacy_rules_path,
        r#"{"redaction_markers":["private:"]}"#,
    );

    let mut transaction = pool.begin().await.expect("transaction should begin");
    TelegramMessageService::save(
        &mut transaction,
        SaveTelegramMessage {
            channel_name: "Example jobs",
            content_text: "A fictional company is hiring a Rust engineer for Aurora tooling.",
            author_name: Some("Example recruiter"),
            published_at: None,
            external_message_id: Some("global-search-1"),
            external_url: Some("https://t.me/example_jobs/global-search-1"),
            content_urls: &[],
        },
    )
    .await
    .expect("Telegram fixture should be saved");

    let gateway = ContextGateway::new(config.clone(), "hermes-test");
    let audit_log = AuditLog::new(config.aurora_home.join("audit/mcp.jsonl"));
    let service = AuroraSearchService::new(gateway, audit_log, "hermes-test");
    let pack = service
        .search(
            &mut transaction,
            SearchAurora {
                query: "Aurora Rust",
                purpose: "find relevant authorized context",
                include_personal_context: true,
                include_telegram: true,
                channel_name: None,
                starts_at: None,
                ends_at: None,
                max_results: 10,
            },
        )
        .await
        .expect("global search should succeed");

    assert!(
        pack.items
            .iter()
            .any(|item| item.source == "aurora://identity-card.md")
    );
    assert!(
        pack.items
            .iter()
            .any(|item| item.source.starts_with("aurora://telegram/messages/"))
    );
    assert!(pack.items.iter().all(|item| !item.source.is_empty()));
    assert!(
        !serde_json::to_string(&pack)
            .unwrap()
            .contains("never disclose")
    );
    let audit = fs::read_to_string(config.aurora_home.join("audit/mcp.jsonl"))
        .expect("global search should be audited");
    assert!(audit.contains("\"tool\":\"search_aurora\""));

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}
