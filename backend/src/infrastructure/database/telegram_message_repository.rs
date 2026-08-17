use crate::domain::search::SearchMatchMode;
use crate::domain::telegram_message::{ContentUrl, TelegramMessage};
use chrono::{DateTime, Utc};
use sqlx::{PgConnection, Postgres, QueryBuilder, types::Json};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewTelegramMessage<'a> {
    pub channel_name: &'a str,
    pub content_text: &'a str,
    pub author_name: Option<&'a str>,
    pub published_at: Option<DateTime<Utc>>,
    pub external_message_id: Option<&'a str>,
    pub external_url: Option<&'a str>,
    pub content_urls: &'a [ContentUrl],
    pub dedup_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveTelegramMessageOutcome {
    Created(TelegramMessage),
    AlreadyExists(TelegramMessage),
}

pub struct TelegramMessageRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTelegramMessages<'a> {
    pub terms: &'a [String],
    pub match_all: bool,
    pub match_mode: SearchMatchMode,
    pub channel_name: Option<&'a str>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub offset: u64,
    pub limit: u32,
}

impl TelegramMessageRepository {
    pub async fn count(
        connection: &mut PgConnection,
        search: &SearchTelegramMessages<'_>,
    ) -> Result<u64, sqlx::Error> {
        if search.terms.is_empty() && !search.match_all {
            return Ok(0);
        }

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM telegram_messages
            WHERE TRUE
            "#,
        );
        if !search.match_all {
            query.push(" AND (");
            push_term_predicates(&mut query, search.terms, search.match_mode);
            query.push(")");
        }
        push_filters(&mut query, search);

        let count: i64 = query.build_query_scalar().fetch_one(connection).await?;
        Ok(count.max(0) as u64)
    }

    pub async fn search(
        connection: &mut PgConnection,
        search: SearchTelegramMessages<'_>,
    ) -> Result<Vec<TelegramMessage>, sqlx::Error> {
        if (search.terms.is_empty() && !search.match_all) || search.limit == 0 {
            return Ok(Vec::new());
        }

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                id,
                channel_name,
                content_text,
                author_name,
                published_at,
                external_message_id,
                external_url,
                content_urls,
                dedup_sha256,
                saved_at
            FROM telegram_messages
            WHERE TRUE
            "#,
        );
        if !search.match_all {
            query.push(" AND (");
            push_term_predicates(&mut query, search.terms, search.match_mode);
            query.push(")");
        }
        push_filters(&mut query, &search);
        query
            .push(" ORDER BY published_at DESC NULLS LAST, saved_at DESC, id DESC OFFSET ")
            .push_bind(search.offset.min(i64::MAX as u64) as i64)
            .push(" LIMIT ")
            .push_bind(i64::from(search.limit));

        query
            .build_query_as::<TelegramMessageRow>()
            .fetch_all(connection)
            .await
            .map(|rows| rows.into_iter().map(TelegramMessage::from).collect())
    }

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
                content_urls,
                dedup_sha256
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (dedup_sha256) DO NOTHING
            RETURNING
                id,
                channel_name,
                content_text,
                author_name,
                published_at,
                external_message_id,
                external_url,
                content_urls,
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
        .bind(Json(new_message.content_urls))
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
                content_urls,
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

fn push_term_predicates<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    terms: &'a [String],
    match_mode: SearchMatchMode,
) {
    for (index, term) in terms.iter().enumerate() {
        if index > 0 {
            query.push(match match_mode {
                SearchMatchMode::AllTerms => " AND ",
                SearchMatchMode::AnyTerms => " OR ",
            });
        }
        let pattern = format!("%{}%", term.to_lowercase());
        query
            .push("(LOWER(content_text) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR LOWER(content_urls::text) LIKE ")
            .push_bind(pattern)
            .push(")");
    }
}

fn push_filters<'a>(query: &mut QueryBuilder<'a, Postgres>, search: &SearchTelegramMessages<'a>) {
    if let Some(channel_name) = search.channel_name {
        query
            .push(" AND channel_name = ")
            .push_bind(channel_name.to_string());
    }
    if let Some(starts_at) = search.starts_at {
        query.push(" AND published_at >= ").push_bind(starts_at);
    }
    if let Some(ends_at) = search.ends_at {
        query.push(" AND published_at < ").push_bind(ends_at);
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
    content_urls: Json<Vec<ContentUrl>>,
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
            content_urls: row.content_urls.0,
            dedup_sha256: row.dedup_sha256,
            saved_at: row.saved_at,
        }
    }
}
