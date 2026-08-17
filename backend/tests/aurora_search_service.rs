use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aurora::application::aurora_search_service::{AuroraSearchService, SearchAurora};
use aurora::application::context_gateway::ContextGateway;
use aurora::application::telegram_message_service::{SaveTelegramMessage, TelegramMessageService};
use aurora::config::AppConfig;
use aurora::domain::search::SearchMatchMode;
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
    TelegramMessageService::save(
        &mut transaction,
        SaveTelegramMessage {
            channel_name: "Example jobs",
            content_text: "Another fictional Aurora Rust role for pagination.",
            author_name: Some("Example recruiter"),
            published_at: None,
            external_message_id: Some("global-search-2"),
            external_url: Some("https://jobs.example.com/global-search-2"),
            content_urls: &[],
        },
    )
    .await
    .expect("second Telegram fixture should be saved");

    let gateway = ContextGateway::new(config.clone(), "hermes-test");
    let audit_log = AuditLog::new(config.aurora_home.join("audit/mcp.jsonl"));
    let service = AuroraSearchService::new(gateway, audit_log, "hermes-test");
    let pack = service
        .search(
            &mut transaction,
            SearchAurora {
                query: Some("Aurora Rust"),
                purpose: "find relevant authorized context",
                match_mode: SearchMatchMode::AllTerms,
                include_personal_context: true,
                include_telegram: true,
                channel_name: None,
                starts_at: None,
                ends_at: None,
                offset: 0,
                page_size: 2,
                count_only: false,
            },
        )
        .await
        .expect("global search should succeed");

    assert_eq!(pack.counts.total_matches, 3);
    assert_eq!(pack.counts.personal_context, 1);
    assert_eq!(pack.counts.telegram, 2);
    assert_eq!(pack.page.returned_count, 2);
    assert!(pack.page.has_more);
    assert_eq!(pack.page.next_cursor.as_deref(), Some("v1:2"));
    assert!(
        pack.items
            .iter()
            .any(|item| item.stored_record_uri == "aurora://identity-card.md")
    );
    let telegram = pack
        .items
        .iter()
        .find(|item| {
            item.stored_record_uri
                .starts_with("aurora://telegram/messages/")
        })
        .expect("Telegram result should be present");
    assert_eq!(telegram.collection_source.platform, "telegram");
    assert_eq!(telegram.collection_source.container_name, "Example jobs");
    assert!(
        ["t.me", "jobs.example.com"]
            .contains(&telegram.original_source.as_ref().unwrap().platform.as_str())
    );
    assert!(
        pack.items
            .iter()
            .all(|item| !item.stored_record_uri.is_empty())
    );

    let second_page = service
        .search(
            &mut transaction,
            SearchAurora {
                query: Some("Aurora Rust"),
                purpose: "continue the authorized search",
                match_mode: SearchMatchMode::AllTerms,
                include_personal_context: true,
                include_telegram: true,
                channel_name: None,
                starts_at: None,
                ends_at: None,
                offset: 2,
                page_size: 2,
                count_only: false,
            },
        )
        .await
        .expect("second search page should succeed");
    assert_eq!(second_page.counts.total_matches, 3);
    assert_eq!(second_page.page.returned_count, 1);
    assert!(!second_page.page.has_more);
    assert!(second_page.page.next_cursor.is_none());

    let count_only = service
        .search(
            &mut transaction,
            SearchAurora {
                query: Some("Aurora Rust"),
                purpose: "count authorized matches",
                match_mode: SearchMatchMode::AllTerms,
                include_personal_context: true,
                include_telegram: true,
                channel_name: None,
                starts_at: None,
                ends_at: None,
                offset: 0,
                page_size: 10,
                count_only: true,
            },
        )
        .await
        .expect("count-only search should succeed");
    assert_eq!(count_only.counts.total_matches, 3);
    assert_eq!(count_only.page.returned_count, 0);
    assert!(count_only.items.is_empty());

    let inventory_count = service
        .search(
            &mut transaction,
            SearchAurora {
                query: None,
                purpose: "count every stored Telegram message",
                match_mode: SearchMatchMode::AllTerms,
                include_personal_context: false,
                include_telegram: true,
                channel_name: None,
                starts_at: None,
                ends_at: None,
                offset: 0,
                page_size: 10,
                count_only: true,
            },
        )
        .await
        .expect("inventory count should succeed without invented keywords");
    assert_eq!(inventory_count.query, None);
    assert_eq!(inventory_count.query_mode, "match_all");
    assert_eq!(inventory_count.counts.telegram, 2);
    assert_eq!(inventory_count.counts.total_matches, 2);
    assert!(inventory_count.items.is_empty());
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
