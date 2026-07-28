use crate::domain::conversation::{Conversation, ConversationKind};
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use std::io::{Error as IoError, ErrorKind};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewConversation<'a> {
    pub import_batch_id: Uuid,
    pub source_conversation_key: &'a str,
    pub title: &'a str,
    pub kind: ConversationKind,
}

pub struct ConversationRepository;

impl ConversationRepository {
    pub async fn create(
        connection: &mut PgConnection,
        new_conversation: NewConversation<'_>,
    ) -> Result<Conversation, sqlx::Error> {
        let row = sqlx::query_as::<_, ConversationRow>(
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
        .bind(new_conversation.import_batch_id)
        .bind(new_conversation.source_conversation_key)
        .bind(new_conversation.title)
        .bind(new_conversation.kind.as_str())
        .fetch_one(connection)
        .await?;

        row.try_into()
    }
}

#[derive(sqlx::FromRow)]
struct ConversationRow {
    id: Uuid,
    import_batch_id: Uuid,
    source_conversation_key: String,
    title: String,
    conversation_kind: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<ConversationRow> for Conversation {
    type Error = sqlx::Error;

    fn try_from(row: ConversationRow) -> Result<Self, Self::Error> {
        let kind = ConversationKind::try_from(row.conversation_kind.as_str())
            .map_err(invalid_domain_value)?;

        Ok(Self {
            id: row.id,
            import_batch_id: row.import_batch_id,
            source_conversation_key: row.source_conversation_key,
            title: row.title,
            kind,
            created_at: row.created_at,
        })
    }
}

fn invalid_domain_value(message: String) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(IoError::new(ErrorKind::InvalidData, message)))
}
