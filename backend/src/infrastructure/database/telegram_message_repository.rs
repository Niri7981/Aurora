use crate::domain::telegram_message::TelegramMessage;
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewTelegramMessage<'a> {
    pub channel_name: &'a str,
    pub content_text: &'a str,
    pub author_name: Option<&'a str>,
    pub published_at: Option<DateTime<Utc>>,
    pub external_message_id: Option<&'a str>,
    pub external_url: Option<&'a str>,
    pub dedup_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveTelegramMessageOutcome {
    Created(TelegramMessage),
    AlreadyExists(TelegramMessage),
}

pub struct TelegramMessageRepository;

impl TelegramMessageRepository {
    pub async fn save(
        connection: &mut PgConnection,
        new_message: NewTelegramMessage<'_>,
    ) -> Result<SaveTelegramMessageOutcome, sqlx::Error> {
        let inserted = sqlx::query_as::<_, TelegramMessageRow>(
            r#"
            INSERT INTO telegram_messages (
                channel_name,
                content_text,
                author_name,
                published_at,
                external_message_id,
                external_url,
                dedup_sha256
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (dedup_sha256) DO NOTHING
            RETURNING
                id,
                channel_name,
                content_text,
                author_name,
                published_at,
                external_message_id,
                external_url,
                dedup_sha256,
                saved_at
            "#,
        )
        .bind(new_message.channel_name)
        .bind(new_message.content_text)
        .bind(new_message.author_name)
        .bind(new_message.published_at)
        .bind(new_message.external_message_id)
        .bind(new_message.external_url)
        .bind(new_message.dedup_sha256)
        .fetch_optional(&mut *connection)
        .await?;

        if let Some(inserted) = inserted {
            return Ok(SaveTelegramMessageOutcome::Created(inserted.into()));
        }

        let existing = Self::find_by_dedup_sha256(connection, new_message.dedup_sha256)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        Ok(SaveTelegramMessageOutcome::AlreadyExists(existing))
    }

    async fn find_by_dedup_sha256(
        connection: &mut PgConnection,
        dedup_sha256: &str,
    ) -> Result<Option<TelegramMessage>, sqlx::Error> {
        sqlx::query_as::<_, TelegramMessageRow>(
            r#"
            SELECT
                id,
                channel_name,
                content_text,
                author_name,
                published_at,
                external_message_id,
                external_url,
                dedup_sha256,
                saved_at
            FROM telegram_messages
            WHERE dedup_sha256 = $1
            "#,
        )
        .bind(dedup_sha256)
        .fetch_optional(connection)
        .await
        .map(|row| row.map(TelegramMessage::from))
    }
}

#[derive(sqlx::FromRow)]
struct TelegramMessageRow {
    id: Uuid,
    channel_name: String,
    content_text: String,
    author_name: Option<String>,
    published_at: Option<DateTime<Utc>>,
    external_message_id: Option<String>,
    external_url: Option<String>,
    dedup_sha256: String,
    saved_at: DateTime<Utc>,
}

impl From<TelegramMessageRow> for TelegramMessage {
    fn from(row: TelegramMessageRow) -> Self {
        Self {
            id: row.id,
            channel_name: row.channel_name,
            content_text: row.content_text,
            author_name: row.author_name,
            published_at: row.published_at,
            external_message_id: row.external_message_id,
            external_url: row.external_url,
            dedup_sha256: row.dedup_sha256,
            saved_at: row.saved_at,
        }
    }
}
