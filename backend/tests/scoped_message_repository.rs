use aurora::domain::conversation::ConversationKind;
use aurora::domain::message::MessageSenderKind;
use aurora::infrastructure::database::analysis_scope_repository::{
    AnalysisScopeRepository, NewAnalysisScope,
};
use aurora::infrastructure::database::conversation_repository::{
    ConversationRepository, NewConversation,
};
use aurora::infrastructure::database::import_batch_repository::{
    CreateImportBatchOutcome, ImportBatchRepository, NewImportBatch,
};
use aurora::infrastructure::database::message_repository::{MessageRepository, NewMessage};
use aurora::infrastructure::database::scoped_message_repository::{
    ReadScopedMessagesError, ScopedMessageRepository,
};
use chrono::{DateTime, Duration, Utc};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn reads_only_active_scope_messages_with_a_stable_cursor() {
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

    let content_sha256 = "2".repeat(64);
    let mut transaction = pool.begin().await.expect("transaction should begin");
    let batch = ImportBatchRepository::create_if_new(
        &mut transaction,
        NewImportBatch {
            source_kind: "aurora_json",
            original_filename: "scoped-read.json",
            content_sha256: &content_sha256,
        },
    )
    .await
    .expect("import batch should be created");
    let batch = match batch {
        CreateImportBatchOutcome::Created(batch) => batch,
        CreateImportBatchOutcome::AlreadyExists(_) => panic!("test batch should be new"),
    };
    let conversation = ConversationRepository::create(
        &mut transaction,
        NewConversation {
            import_batch_id: batch.id,
            source_conversation_key: "wechat:contact:scoped-read",
            title: "分页读取测试",
            kind: ConversationKind::Direct,
        },
    )
    .await
    .expect("conversation should be created");

    for (source_sequence, sender_kind, sender_key, sender_name, sent_at, content) in [
        (
            0,
            MessageSenderKind::Participant,
            "contact:test",
            "测试联系人",
            timestamp("2024-12-31T23:59:59Z"),
            "范围之前",
        ),
        (
            1,
            MessageSenderKind::SelfUser,
            "self",
            "我",
            timestamp("2025-01-10T10:00:00Z"),
            "范围内第一条",
        ),
        (
            2,
            MessageSenderKind::Participant,
            "contact:test",
            "测试联系人",
            timestamp("2025-01-10T10:00:00Z"),
            "同一时间第二条",
        ),
        (
            3,
            MessageSenderKind::SelfUser,
            "self",
            "我",
            timestamp("2025-01-20T10:00:00Z"),
            "范围内第三条",
        ),
        (
            4,
            MessageSenderKind::Participant,
            "contact:test",
            "测试联系人",
            timestamp("2025-02-01T00:00:00Z"),
            "结束边界",
        ),
    ] {
        MessageRepository::create(
            &mut transaction,
            NewMessage {
                conversation_id: conversation.id,
                source_sequence,
                sender_kind,
                sender_key,
                sender_display_name: sender_name,
                sent_at,
                content_text: content,
            },
        )
        .await
        .expect("message should be created");
    }

    let scope = AnalysisScopeRepository::create(
        &mut transaction,
        NewAnalysisScope {
            conversation_id: conversation.id,
            starts_at: timestamp("2025-01-01T00:00:00Z"),
            ends_at: timestamp("2025-02-01T00:00:00Z"),
            purpose: "测试安全分页",
            expires_at: Utc::now() + Duration::hours(1),
        },
    )
    .await
    .expect("analysis scope should be created");

    let first_page = ScopedMessageRepository::read_page(&mut transaction, scope.id, None, 2)
        .await
        .expect("first page should be readable");
    assert_eq!(
        first_page
            .messages
            .iter()
            .map(|message| message.source_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let cursor = first_page.next_cursor.expect("first page should continue");

    let second_page =
        ScopedMessageRepository::read_page(&mut transaction, scope.id, Some(cursor), 2)
            .await
            .expect("second page should be readable");
    assert_eq!(
        second_page
            .messages
            .iter()
            .map(|message| message.source_sequence)
            .collect::<Vec<_>>(),
        vec![3]
    );
    assert_eq!(second_page.next_cursor, None);

    sqlx::query("UPDATE analysis_scopes SET revoked_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(scope.id)
        .execute(&mut *transaction)
        .await
        .expect("scope should be revoked");
    let revoked = ScopedMessageRepository::read_page(&mut transaction, scope.id, None, 2)
        .await
        .expect_err("revoked scope must not disclose messages");
    assert!(matches!(revoked, ReadScopedMessagesError::ScopeUnavailable));

    let invalid_limit = ScopedMessageRepository::read_page(&mut transaction, scope.id, None, 0)
        .await
        .expect_err("zero page size should be rejected");
    assert!(matches!(
        invalid_limit,
        ReadScopedMessagesError::InvalidPageSize(0)
    ));

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("test timestamp should be valid")
        .with_timezone(&Utc)
}
