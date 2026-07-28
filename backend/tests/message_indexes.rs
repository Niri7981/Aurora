use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn migration_creates_scope_ordered_message_index() {
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

    let index_definition: String = sqlx::query_scalar(
        r#"
        SELECT indexdef
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'messages'
          AND indexname = 'messages_conversation_sent_at_sequence_idx'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("message scope index should exist");

    assert!(
        index_definition.contains("(conversation_id, sent_at, source_sequence)"),
        "unexpected index definition: {index_definition}"
    );
}
