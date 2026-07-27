use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportBatch {
    pub id: Uuid,
    pub source_kind: String,
    pub original_filename: String,
    pub content_sha256: String,
    pub imported_at: DateTime<Utc>,
}
