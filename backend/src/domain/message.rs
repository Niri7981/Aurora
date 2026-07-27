use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub source_sequence: i64,
    pub sender_key: String,
    pub sender_display_name: String,
    pub sent_at: DateTime<Utc>,
    pub content_text: String,
    pub created_at: DateTime<Utc>,
}
