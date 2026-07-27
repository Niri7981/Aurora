use aurora::domain::import_batch::ImportBatch;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn migration_creates_import_batches_and_rejects_duplicate_files() {
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
    let content_sha256 = "a".repeat(64);
    let batch = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            String,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        INSERT INTO import_batches (source_kind, original_filename, content_sha256)
        VALUES ($1, $2, $3)
        RETURNING id, source_kind, original_filename, content_sha256, imported_at
        "#,
    )
    .bind("aurora_json")
    .bind("wechat-chat.json")
    .bind(&content_sha256)
    .fetch_one(&mut *transaction)
    .await
    .map(
        |(id, source_kind, original_filename, content_sha256, imported_at)| ImportBatch {
            id,
            source_kind,
            original_filename,
            content_sha256,
            imported_at,
        },
    )
    .expect("import batch should be inserted");

    assert_eq!(batch.source_kind, "aurora_json");
    assert_eq!(batch.original_filename, "wechat-chat.json");
    assert_eq!(batch.content_sha256, content_sha256);

    let duplicate = sqlx::query(
        r#"
        INSERT INTO import_batches (source_kind, original_filename, content_sha256)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind("aurora_json")
    .bind("same-file-again.json")
    .bind(&content_sha256)
    .execute(&mut *transaction)
    .await
    .expect_err("duplicate file hash should be rejected");

    assert_eq!(
        duplicate
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some("import_batches_content_sha256_unique")
    );

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}
