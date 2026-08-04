use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aurora::application::profile_update_proposal_service::ProfileUpdateProposalService;
use aurora::config::AppConfig;
use aurora::domain::profile_update_proposal::{ProfileUpdateStatus, ProfileUpdateTarget};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);
const TEST_CLIENT: &str = "profile-proposal-service-test";

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn creates_a_pending_proposal_from_server_owned_profile_state() {
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

    let config = test_config();
    let current_content = "# Current focus\n\nBuild the MCP proposal tool.\n";
    write_file(&config.current_focus_path, current_content);
    write_file(&config.preferences_path, r#"{"response_style":"concise"}"#);
    let service = ProfileUpdateProposalService::new(config, TEST_CLIENT);
    let mut transaction = pool.begin().await.expect("transaction should begin");
    let proposals_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM profile_update_proposals WHERE proposed_by = $1")
            .bind(TEST_CLIENT)
            .fetch_one(&mut *transaction)
            .await
            .expect("initial proposal count should be readable");

    let proposed_content = "# Current focus\n\nReview profile updates safely.\n";
    let proposal = service
        .propose(
            &mut transaction,
            ProfileUpdateTarget::CurrentFocus,
            proposed_content,
            "  The user's active milestone changed  ",
        )
        .await
        .expect("proposal should be created");

    assert_eq!(proposal.target, ProfileUpdateTarget::CurrentFocus);
    assert_eq!(proposal.proposed_content, proposed_content);
    assert_eq!(proposal.reason, "The user's active milestone changed");
    assert_eq!(proposal.proposed_by, TEST_CLIENT);
    assert_eq!(proposal.status, ProfileUpdateStatus::Pending);
    assert_eq!(
        proposal.base_sha256,
        format!("{:x}", Sha256::digest(current_content.as_bytes()))
    );

    let found = service
        .find_by_id(&mut transaction, proposal.id)
        .await
        .expect("proposal lookup should succeed")
        .expect("created proposal should be found");
    assert_eq!(found, proposal);

    let pending = service
        .list_pending(&mut transaction)
        .await
        .expect("pending proposals should be listed");
    assert!(pending.iter().any(|item| item.id == proposal.id));

    let invalid_preferences = service
        .propose(
            &mut transaction,
            ProfileUpdateTarget::Preferences,
            "not-json",
            "Change a preference",
        )
        .await
        .expect_err("invalid preference JSON should be rejected before insertion");
    assert!(invalid_preferences.contains("must be valid JSON"));

    let proposals_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM profile_update_proposals WHERE proposed_by = $1")
            .bind(TEST_CLIENT)
            .fetch_one(&mut *transaction)
            .await
            .expect("final proposal count should be readable");
    assert_eq!(proposals_after, proposals_before + 1);

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}

fn test_config() -> AppConfig {
    let root = unique_temp_dir("profile-proposal-service");
    AppConfig {
        workspace: root.clone(),
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
