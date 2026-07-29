use crate::domain::message::{Message, MessageSenderKind};
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use std::error::Error;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub const MAX_MESSAGE_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageCursor {
    pub sent_at: DateTime<Utc>,
    pub source_sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedMessagePage {
    pub messages: Vec<Message>,
    pub next_cursor: Option<MessageCursor>,
}

#[derive(Debug)]
pub enum ReadScopedMessagesError {
    InvalidPageSize(u32),
    ScopeUnavailable,
    InvalidStoredMessage(String),
    Database(sqlx::Error),
}

impl Display for ReadScopedMessagesError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPageSize(limit) => write!(
                formatter,
                "message page size must be between 1 and {MAX_MESSAGE_PAGE_SIZE}, got {limit}"
            ),
            Self::ScopeUnavailable => {
                write!(formatter, "analysis scope is missing, expired, or revoked")
            }
            Self::InvalidStoredMessage(message) => formatter.write_str(message),
            Self::Database(error) => write!(formatter, "failed to read scoped messages: {error}"),
        }
    }
}

impl Error for ReadScopedMessagesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for ReadScopedMessagesError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

pub struct ScopedMessageRepository;

impl ScopedMessageRepository {
    pub async fn read_page(
        connection: &mut PgConnection,
        scope_id: Uuid,
        after: Option<MessageCursor>,
        limit: u32,
    ) -> Result<ScopedMessagePage, ReadScopedMessagesError> {
        if !(1..=MAX_MESSAGE_PAGE_SIZE).contains(&limit) {
            return Err(ReadScopedMessagesError::InvalidPageSize(limit));
        }

        let (cursor_sent_at, cursor_sequence) = match after {
            Some(cursor) => (Some(cursor.sent_at), Some(cursor.source_sequence)),
            None => (None, None),
        };
        let fetch_limit = i64::from(limit) + 1;
        let checked_at = Utc::now();

        let rows = sqlx::query_as::<_, ScopedMessageRow>(
            r#"
            WITH active_scope AS (
                SELECT id, conversation_id, starts_at, ends_at
                FROM analysis_scopes
                WHERE id = $1
                  AND created_at <= $2
                  AND $2 < expires_at
                  AND (revoked_at IS NULL OR $2 < revoked_at)
            )
            SELECT
                active_scope.id AS scope_id,
                page.id AS message_id,
                page.conversation_id,
                page.source_sequence,
                page.sender_kind,
                page.sender_key,
                page.sender_display_name,
                page.sent_at,
                page.content_text,
                page.created_at
            FROM active_scope
            LEFT JOIN LATERAL (
                SELECT
                    messages.id,
                    messages.conversation_id,
                    messages.source_sequence,
                    messages.sender_kind,
                    messages.sender_key,
                    messages.sender_display_name,
                    messages.sent_at,
                    messages.content_text,
                    messages.created_at
                FROM messages
                WHERE messages.conversation_id = active_scope.conversation_id
                  AND active_scope.starts_at <= messages.sent_at
                  AND messages.sent_at < active_scope.ends_at
                  AND (
                      $3::timestamptz IS NULL
                      OR (messages.sent_at, messages.source_sequence) >
                         ($3::timestamptz, $4::bigint)
                  )
                ORDER BY messages.sent_at, messages.source_sequence
                LIMIT $5
            ) AS page ON TRUE
            ORDER BY page.sent_at NULLS LAST, page.source_sequence NULLS LAST
            "#,
        )
        .bind(scope_id)
        .bind(checked_at)
        .bind(cursor_sent_at)
        .bind(cursor_sequence)
        .bind(fetch_limit)
        .fetch_all(connection)
        .await?;

        if rows.is_empty() {
            return Err(ReadScopedMessagesError::ScopeUnavailable);
        }

        let mut messages = rows
            .into_iter()
            .filter_map(ScopedMessageRow::into_message)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = messages.len() > limit as usize;
        messages.truncate(limit as usize);
        let next_cursor = has_more.then(|| {
            let last = messages
                .last()
                .expect("a page with more messages must contain one message");
            MessageCursor {
                sent_at: last.sent_at,
                source_sequence: last.source_sequence,
            }
        });

        Ok(ScopedMessagePage {
            messages,
            next_cursor,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ScopedMessageRow {
    #[allow(dead_code)]
    scope_id: Uuid,
    message_id: Option<Uuid>,
    conversation_id: Option<Uuid>,
    source_sequence: Option<i64>,
    sender_kind: Option<String>,
    sender_key: Option<String>,
    sender_display_name: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    content_text: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

impl ScopedMessageRow {
    fn into_message(self) -> Option<Result<Message, ReadScopedMessagesError>> {
        let id = self.message_id?;
        Some(self.complete_message(id))
    }

    fn complete_message(self, id: Uuid) -> Result<Message, ReadScopedMessagesError> {
        let missing = |field: &str| {
            ReadScopedMessagesError::InvalidStoredMessage(format!(
                "scoped message {id} is missing {field}"
            ))
        };
        let sender_kind = self
            .sender_kind
            .ok_or_else(|| missing("sender_kind"))
            .and_then(|value| {
                MessageSenderKind::try_from(value.as_str())
                    .map_err(ReadScopedMessagesError::InvalidStoredMessage)
            })?;

        Ok(Message {
            id,
            conversation_id: self
                .conversation_id
                .ok_or_else(|| missing("conversation_id"))?,
            source_sequence: self
                .source_sequence
                .ok_or_else(|| missing("source_sequence"))?,
            sender_kind,
            sender_key: self.sender_key.ok_or_else(|| missing("sender_key"))?,
            sender_display_name: self
                .sender_display_name
                .ok_or_else(|| missing("sender_display_name"))?,
            sent_at: self.sent_at.ok_or_else(|| missing("sent_at"))?,
            content_text: self.content_text.ok_or_else(|| missing("content_text"))?,
            created_at: self.created_at.ok_or_else(|| missing("created_at"))?,
        })
    }
}
