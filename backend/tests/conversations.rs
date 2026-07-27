use aurora::domain::conversation::{Conversation, ConversationKind};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn migration_creates_conversations_and_enforces_source_identity() {
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
    .bind("conversation-test.json")
    .bind("b".repeat(64))
    .fetch_one(&mut *transaction)
    .await
    .expect("import batch should be inserted");

    let row = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            uuid::Uuid,
            String,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        INSERT INTO conversations (
            import_batch_id,
            source_conversation_key,
            title,
            conversation_kind
        )
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            import_batch_id,
            source_conversation_key,
            title,
            conversation_kind,
            created_at
        "#,
    )
    .bind(import_batch_id)
    .bind("wechat:contact:zhang-san")
    .bind("我和张三")
    .bind(ConversationKind::Direct.as_str())
    .fetch_one(&mut *transaction)
    .await
    .expect("conversation should be inserted");

    let conversation = Conversation {
        id: row.0,
        import_batch_id: row.1,
        source_conversation_key: row.2,
        title: row.3,
        kind: ConversationKind::try_from(row.4.as_str())
            .expect("stored conversation kind should be valid"),
        created_at: row.5,
    };

    assert_eq!(conversation.import_batch_id, import_batch_id);
    assert_eq!(
        conversation.source_conversation_key,
        "wechat:contact:zhang-san"
    );
    assert_eq!(conversation.title, "我和张三");
    assert_eq!(conversation.kind, ConversationKind::Direct);

    let duplicate = sqlx::query(
        r#"
        INSERT INTO conversations (
            import_batch_id,
            source_conversation_key,
            title,
            conversation_kind
        )
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(import_batch_id)
    .bind("wechat:contact:zhang-san")
    .bind("重复会话")
    .bind(ConversationKind::Direct.as_str())
    .execute(&mut *transaction)
    .await
    .expect_err("duplicate conversation key in one import should be rejected");

    assert_eq!(
        duplicate
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some("conversations_batch_source_key_unique")
    );

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}
