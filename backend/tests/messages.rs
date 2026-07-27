use aurora::domain::message::{Message, MessageSenderKind};
use sqlx::postgres::PgPoolOptions;

type MessageRow = (
    uuid::Uuid,
    uuid::Uuid,
    i64,
    String,
    String,
    String,
    chrono::DateTime<chrono::Utc>,
    String,
    chrono::DateTime<chrono::Utc>,
);

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn migration_creates_ordered_messages_without_rewriting_content() {
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
    .bind("message-test.json")
    .bind("c".repeat(64))
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
    .bind("wechat:contact:li-si")
    .bind("我和李四")
    .bind("direct")
    .fetch_one(&mut *transaction)
    .await
    .expect("conversation should be inserted");

    let sent_at = chrono::DateTime::parse_from_rfc3339("2026-07-27T10:00:00+08:00")
        .expect("timestamp should be valid")
        .with_timezone(&chrono::Utc);

    for (source_sequence, sender_kind, sender_key, sender_display_name, content_text) in [
        (
            1_i64,
            MessageSenderKind::Participant,
            "contact:li-si",
            "李四",
            "第二条",
        ),
        (
            0_i64,
            MessageSenderKind::SelfUser,
            "self",
            "我",
            "  第一条原文保留空格  ",
        ),
    ] {
        sqlx::query(
            r#"
            INSERT INTO messages (
                conversation_id,
                source_sequence,
                sender_kind,
                sender_key,
                sender_display_name,
                sent_at,
                content_text
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(conversation_id)
        .bind(source_sequence)
        .bind(sender_kind.as_str())
        .bind(sender_key)
        .bind(sender_display_name)
        .bind(sent_at)
        .bind(content_text)
        .execute(&mut *transaction)
        .await
        .expect("message should be inserted");
    }

    let rows = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            id,
            conversation_id,
            source_sequence,
            sender_kind,
            sender_key,
            sender_display_name,
            sent_at,
            content_text,
            created_at
        FROM messages
        WHERE conversation_id = $1
        ORDER BY source_sequence
        "#,
    )
    .bind(conversation_id)
    .fetch_all(&mut *transaction)
    .await
    .expect("messages should be readable in source order");

    let messages: Vec<Message> = rows
        .into_iter()
        .map(|row| Message {
            id: row.0,
            conversation_id: row.1,
            source_sequence: row.2,
            sender_kind: MessageSenderKind::try_from(row.3.as_str())
                .expect("stored sender kind should be valid"),
            sender_key: row.4,
            sender_display_name: row.5,
            sent_at: row.6,
            content_text: row.7,
            created_at: row.8,
        })
        .collect();

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].source_sequence, 0);
    assert_eq!(messages[0].sender_kind, MessageSenderKind::SelfUser);
    assert_eq!(messages[0].sender_key, "self");
    assert_eq!(messages[0].sender_display_name, "我");
    assert_eq!(messages[0].content_text, "  第一条原文保留空格  ");
    assert_eq!(messages[1].source_sequence, 1);
    assert_eq!(messages[1].sender_kind, MessageSenderKind::Participant);
    assert_eq!(messages[1].content_text, "第二条");

    let duplicate = sqlx::query(
        r#"
        INSERT INTO messages (
            conversation_id,
            source_sequence,
            sender_kind,
            sender_key,
            sender_display_name,
            sent_at,
            content_text
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(conversation_id)
    .bind(0_i64)
    .bind(MessageSenderKind::Participant.as_str())
    .bind("contact:li-si")
    .bind("李四")
    .bind(sent_at)
    .bind("重复顺序")
    .execute(&mut *transaction)
    .await
    .expect_err("duplicate source sequence should be rejected");

    assert_eq!(
        duplicate
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some("messages_conversation_source_sequence_unique")
    );

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}
