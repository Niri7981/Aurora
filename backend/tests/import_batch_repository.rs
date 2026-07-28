use aurora::infrastructure::database::import_batch_repository::{
    CreateImportBatchOutcome, ImportBatchRepository, NewImportBatch,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn creates_one_import_batch_per_file_hash_without_committing_the_transaction() {
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

    let content_sha256 = "e".repeat(64);
    let mut transaction = pool.begin().await.expect("transaction should begin");

    let created = ImportBatchRepository::create_if_new(
        &mut transaction,
        NewImportBatch {
            source_kind: "aurora_json",
            original_filename: "wechat-chat.json",
            content_sha256: &content_sha256,
        },
    )
    .await
    .expect("first import batch should be created");

    let created_batch = match created {
        CreateImportBatchOutcome::Created(batch) => batch,
        CreateImportBatchOutcome::AlreadyExists(_) => {
            panic!("first import should not already exist")
        }
    };

    let duplicate = ImportBatchRepository::create_if_new(
        &mut transaction,
        NewImportBatch {
            source_kind: "aurora_json",
            original_filename: "renamed-copy.json",
            content_sha256: &content_sha256,
        },
    )
    .await
    .expect("duplicate hash should return the existing batch");

    let existing_batch = match duplicate {
        CreateImportBatchOutcome::Created(_) => panic!("duplicate hash should not be inserted"),
        CreateImportBatchOutcome::AlreadyExists(batch) => batch,
    };

    assert_eq!(existing_batch.id, created_batch.id);
    assert_eq!(existing_batch.original_filename, "wechat-chat.json");
    assert_eq!(existing_batch.content_sha256, content_sha256);

    let found = ImportBatchRepository::find_by_content_sha256(
        &mut transaction,
        &existing_batch.content_sha256,
    )
    .await
    .expect("hash lookup should succeed")
    .expect("created batch should be found");
    assert_eq!(found, created_batch);

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM import_batches WHERE content_sha256 = $1")
            .bind(&content_sha256)
            .fetch_one(&pool)
            .await
            .expect("rolled back batch count should be readable");
    assert_eq!(remaining, 0);
}
