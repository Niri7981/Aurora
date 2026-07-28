use crate::domain::import_batch::ImportBatch;
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewImportBatch<'a> {
    pub source_kind: &'a str,
    pub original_filename: &'a str,
    pub content_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateImportBatchOutcome {
    Created(ImportBatch),
    AlreadyExists(ImportBatch),
}

pub struct ImportBatchRepository;

impl ImportBatchRepository {
    pub async fn create_if_new(
        connection: &mut PgConnection,
        new_batch: NewImportBatch<'_>,
    ) -> Result<CreateImportBatchOutcome, sqlx::Error> {
        let inserted = sqlx::query_as::<_, ImportBatchRow>(
            r#"
            INSERT INTO import_batches (source_kind, original_filename, content_sha256)
            VALUES ($1, $2, $3)
            ON CONFLICT (content_sha256) DO NOTHING
            RETURNING id, source_kind, original_filename, content_sha256, imported_at
            "#,
        )
        .bind(new_batch.source_kind)
        .bind(new_batch.original_filename)
        .bind(new_batch.content_sha256)
        .fetch_optional(&mut *connection)
        .await?;

        if let Some(inserted) = inserted {
            return Ok(CreateImportBatchOutcome::Created(inserted.into()));
        }

        let existing = Self::find_by_content_sha256(connection, new_batch.content_sha256)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        Ok(CreateImportBatchOutcome::AlreadyExists(existing))
    }

    pub async fn find_by_content_sha256(
        connection: &mut PgConnection,
        content_sha256: &str,
    ) -> Result<Option<ImportBatch>, sqlx::Error> {
        sqlx::query_as::<_, ImportBatchRow>(
            r#"
            SELECT id, source_kind, original_filename, content_sha256, imported_at
            FROM import_batches
            WHERE content_sha256 = $1
            "#,
        )
        .bind(content_sha256)
        .fetch_optional(connection)
        .await
        .map(|row| row.map(ImportBatch::from))
    }
}

#[derive(sqlx::FromRow)]
struct ImportBatchRow {
    id: Uuid,
    source_kind: String,
    original_filename: String,
    content_sha256: String,
    imported_at: DateTime<Utc>,
}

impl From<ImportBatchRow> for ImportBatch {
    fn from(row: ImportBatchRow) -> Self {
        Self {
            id: row.id,
            source_kind: row.source_kind,
            original_filename: row.original_filename,
            content_sha256: row.content_sha256,
            imported_at: row.imported_at,
        }
    }
}
