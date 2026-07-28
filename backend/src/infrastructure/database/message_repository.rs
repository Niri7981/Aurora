use crate::domain::message::{Message, MessageSenderKind};
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use std::io::{Error as IoError, ErrorKind};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewMessage<'a> {
    pub conversation_id: Uuid,
    pub source_sequence: i64,
    pub sender_kind: MessageSenderKind,
    pub sender_key: &'a str,
    pub sender_display_name: &'a str,
    pub sent_at: DateTime<Utc>,
    pub content_text: &'a str,
}

pub struct MessageRepository;

impl MessageRepository {
    pub async fn create(
        connection: &mut PgConnection,
        new_message: NewMessage<'_>,
    ) -> Result<Message, sqlx::Error> {
        let row = sqlx::query_as::<_, MessageRow>(
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
            RETURNING
                id,
                conversation_id,
                source_sequence,
                sender_kind,
                sender_key,
                sender_display_name,
                sent_at,
                content_text,
                created_at
            "#,
        )
        .bind(new_message.conversation_id)
        .bind(new_message.source_sequence)
        .bind(new_message.sender_kind.as_str())
        .bind(new_message.sender_key)
        .bind(new_message.sender_display_name)
        .bind(new_message.sent_at)
        .bind(new_message.content_text)
        .fetch_one(connection)
        .await?;

        row.try_into()
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    conversation_id: Uuid,
    source_sequence: i64,
    sender_kind: String,
    sender_key: String,
    sender_display_name: String,
    sent_at: DateTime<Utc>,
    content_text: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<MessageRow> for Message {
    type Error = sqlx::Error;

    fn try_from(row: MessageRow) -> Result<Self, Self::Error> {
        let sender_kind =
            MessageSenderKind::try_from(row.sender_kind.as_str()).map_err(invalid_domain_value)?;

        Ok(Self {
            id: row.id,
            conversation_id: row.conversation_id,
            source_sequence: row.source_sequence,
            sender_kind,
            sender_key: row.sender_key,
            sender_display_name: row.sender_display_name,
            sent_at: row.sent_at,
            content_text: row.content_text,
            created_at: row.created_at,
        })
    }
}

fn invalid_domain_value(message: String) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(IoError::new(ErrorKind::InvalidData, message)))
}
