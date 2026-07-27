use aurora::domain::analysis_scope::AnalysisScope;
use sqlx::postgres::PgPoolOptions;

type AnalysisScopeRow = (
    uuid::Uuid,
    uuid::Uuid,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    String,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    Option<chrono::DateTime<chrono::Utc>>,
);

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn migration_creates_bounded_expiring_analysis_scopes() {
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

    let mut transaction = pool.begin().await.expect("transaction should begin");
    let import_batch_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO import_batches (source_kind, original_filename, content_sha256)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind("aurora_json")
    .bind("analysis-scope-test.json")
    .bind("d".repeat(64))
    .fetch_one(&mut *transaction)
    .await
    .expect("import batch should be inserted");

    let conversation_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO conversations (
            import_batch_id,
            source_conversation_key,
            title,
            conversation_kind
        )
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(import_batch_id)
    .bind("wechat:contact:wang-wu")
    .bind("我和王五")
    .bind("direct")
    .fetch_one(&mut *transaction)
    .await
    .expect("conversation should be inserted");

    let starts_at = timestamp("2025-01-01T00:00:00Z");
    let ends_at = timestamp("2025-02-01T00:00:00Z");
    let created_at = timestamp("2026-07-27T00:00:00Z");
    let expires_at = timestamp("2026-07-28T00:00:00Z");

    let row = sqlx::query_as::<_, AnalysisScopeRow>(
        r#"
        INSERT INTO analysis_scopes (
            conversation_id,
            starts_at,
            ends_at,
            purpose,
            created_at,
            expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            conversation_id,
            starts_at,
            ends_at,
            purpose,
            created_at,
            expires_at,
            revoked_at
        "#,
    )
    .bind(conversation_id)
    .bind(starts_at)
    .bind(ends_at)
    .bind("分析关系变化")
    .bind(created_at)
    .bind(expires_at)
    .fetch_one(&mut *transaction)
    .await
    .expect("analysis scope should be inserted");

    let scope = AnalysisScope {
        id: row.0,
        conversation_id: row.1,
        starts_at: row.2,
        ends_at: row.3,
        purpose: row.4,
        created_at: row.5,
        expires_at: row.6,
        revoked_at: row.7,
    };

    assert_eq!(scope.conversation_id, conversation_id);
    assert_eq!(scope.purpose, "分析关系变化");
    assert!(scope.contains_message_at(starts_at));
    assert!(!scope.contains_message_at(ends_at));
    assert!(scope.is_active_at(timestamp("2026-07-27T12:00:00Z")));

    let invalid_range = sqlx::query(
        r#"
        INSERT INTO analysis_scopes (
            conversation_id,
            starts_at,
            ends_at,
            purpose,
            created_at,
            expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(conversation_id)
    .bind(ends_at)
    .bind(starts_at)
    .bind("无效时间范围")
    .bind(created_at)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await
    .expect_err("reversed analysis range should be rejected");

    assert_eq!(
        invalid_range
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some("analysis_scopes_time_range_check")
    );

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}

fn timestamp(value: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(value)
        .expect("test timestamp should be valid")
        .with_timezone(&chrono::Utc)
}
