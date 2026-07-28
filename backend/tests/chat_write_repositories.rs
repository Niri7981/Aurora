use aurora::domain::conversation::ConversationKind;
use aurora::domain::message::MessageSenderKind;
use aurora::infrastructure::database::conversation_repository::{
    ConversationRepository, NewConversation,
};
use aurora::infrastructure::database::import_batch_repository::{
    CreateImportBatchOutcome, ImportBatchRepository, NewImportBatch,
};
use aurora::infrastructure::database::message_repository::{MessageRepository, NewMessage};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn saves_a_conversation_and_messages_in_one_caller_owned_transaction() {
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

    let content_sha256 = "f".repeat(64);
    let mut transaction = pool.begin().await.expect("transaction should begin");

    let batch = ImportBatchRepository::create_if_new(
        &mut transaction,
        NewImportBatch {
            source_kind: "aurora_json",
            original_filename: "repository-chat.json",
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
            source_conversation_key: "wechat:contact:zhao-liu",
            title: "我和赵六",
            kind: ConversationKind::Direct,
        },
    )
    .await
    .expect("conversation should be created");

    let sent_at = chrono::DateTime::parse_from_rfc3339("2025-01-01T10:00:00+08:00")
        .expect("timestamp should be valid")
        .with_timezone(&chrono::Utc);

    let first = MessageRepository::create(
        &mut transaction,
        NewMessage {
            conversation_id: conversation.id,
            source_sequence: 0,
            sender_kind: MessageSenderKind::SelfUser,
            sender_key: "self",
            sender_display_name: "我",
            sent_at,
            content_text: "  原文保留空格  ",
        },
    )
    .await
    .expect("first message should be created");

    let second = MessageRepository::create(
        &mut transaction,
        NewMessage {
            conversation_id: conversation.id,
            source_sequence: 1,
            sender_kind: MessageSenderKind::Participant,
            sender_key: "contact:zhao-liu",
            sender_display_name: "赵六",
            sent_at,
            content_text: "第二条",
        },
    )
    .await
    .expect("second message should be created");

    assert_eq!(conversation.import_batch_id, batch.id);
    assert_eq!(conversation.kind, ConversationKind::Direct);
    assert_eq!(first.conversation_id, conversation.id);
    assert_eq!(first.sender_kind, MessageSenderKind::SelfUser);
    assert_eq!(first.content_text, "  原文保留空格  ");
    assert_eq!(second.source_sequence, 1);
    assert_eq!(second.sender_display_name, "赵六");

    let source_batch_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        SELECT import_batches.id
        FROM messages
        JOIN conversations ON conversations.id = messages.conversation_id
        JOIN import_batches ON import_batches.id = conversations.import_batch_id
        WHERE messages.id = $1
        "#,
    )
    .bind(second.id)
    .fetch_one(&mut *transaction)
    .await
    .expect("message provenance should be queryable");
    assert_eq!(source_batch_id, batch.id);

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");

    let remaining_batches: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM import_batches WHERE content_sha256 = $1")
            .bind(&content_sha256)
            .fetch_one(&pool)
            .await
            .expect("rolled back batch count should be readable");
    assert_eq!(remaining_batches, 0);
}
