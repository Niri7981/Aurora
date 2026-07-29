use aurora::domain::conversation::ConversationKind;
use aurora::infrastructure::database::analysis_scope_repository::{
    AnalysisScopeRepository, NewAnalysisScope,
};
use aurora::infrastructure::database::conversation_repository::{
    ConversationRepository, NewConversation,
};
use aurora::infrastructure::database::import_batch_repository::{
    CreateImportBatchOutcome, ImportBatchRepository, NewImportBatch,
};
use chrono::{Duration, Utc};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn creates_a_bounded_scope_without_committing_the_transaction() {
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

    let content_sha256 = "1".repeat(64);
    let mut transaction = pool.begin().await.expect("transaction should begin");

    let batch = ImportBatchRepository::create_if_new(
        &mut transaction,
        NewImportBatch {
            source_kind: "aurora_json",
            original_filename: "scope-repository.json",
            content_sha256: &content_sha256,
        },
    )
    .await
    .expect("import batch should be created");
    let batch = match batch {
        CreateImportBatchOutcome::Created(batch) => batch,
        CreateImportBatchOutcome::AlreadyExists(_) => {
            panic!("test import batch should be new")
        }
    };

    let conversation = ConversationRepository::create(
        &mut transaction,
        NewConversation {
            import_batch_id: batch.id,
            source_conversation_key: "wechat:contact:scope-test",
            title: "授权测试会话",
            kind: ConversationKind::Direct,
        },
    )
    .await
    .expect("conversation should be created");

    let starts_at = timestamp("2025-01-01T00:00:00Z");
    let ends_at = timestamp("2025-02-01T00:00:00Z");
    let expires_at = Utc::now() + Duration::hours(1);
    let scope = AnalysisScopeRepository::create(
        &mut transaction,
        NewAnalysisScope {
            conversation_id: conversation.id,
            starts_at,
            ends_at,
            purpose: "分析关系变化",
            expires_at,
        },
    )
    .await
    .expect("analysis scope should be created");

    assert_eq!(scope.conversation_id, conversation.id);
    assert_eq!(scope.starts_at, starts_at);
    assert_eq!(scope.ends_at, ends_at);
    assert_eq!(scope.purpose, "分析关系变化");
    assert_eq!(scope.expires_at, expires_at);
    assert_eq!(scope.revoked_at, None);
    assert!(scope.contains_message_at(starts_at));
    assert!(!scope.contains_message_at(ends_at));
    assert!(scope.is_active_at(scope.created_at + Duration::minutes(1)));

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_scopes WHERE id = $1")
        .bind(scope.id)
        .fetch_one(&pool)
        .await
        .expect("rolled back scope count should be readable");
    assert_eq!(remaining, 0);
}

fn timestamp(value: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(value)
        .expect("test timestamp should be valid")
        .with_timezone(&Utc)
}
