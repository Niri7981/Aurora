use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContentUrl {
    pub url: String,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramMessage {
    pub id: Uuid,
    pub channel_name: String,
    pub content_text: String,
    pub author_name: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub external_message_id: Option<String>,
    pub external_url: Option<String>,
    pub content_urls: Vec<ContentUrl>,
    pub dedup_sha256: String,
    pub saved_at: DateTime<Utc>,
}
