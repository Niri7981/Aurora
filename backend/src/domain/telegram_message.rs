use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramMessage {
    pub id: Uuid,
    pub channel_name: String,
    pub content_text: String,
    pub author_name: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub external_message_id: Option<String>,
    pub external_url: Option<String>,
    pub dedup_sha256: String,
    pub saved_at: DateTime<Utc>,
}
