use crate::domain::analysis_scope::AnalysisScope;
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewAnalysisScope<'a> {
    pub conversation_id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub purpose: &'a str,
    pub expires_at: DateTime<Utc>,
}

pub struct AnalysisScopeRepository;

impl AnalysisScopeRepository {
    pub async fn create(
        connection: &mut PgConnection,
        new_scope: NewAnalysisScope<'_>,
    ) -> Result<AnalysisScope, sqlx::Error> {
        sqlx::query_as::<_, AnalysisScopeRow>(
            r#"
            INSERT INTO analysis_scopes (
                conversation_id,
                starts_at,
                ends_at,
                purpose,
                expires_at
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id,
                conversation_id,
                starts_at,
                ends_at,
                purpose,
                created_at,
                expires_at,
                revoked_at
            "#,
        )
        .bind(new_scope.conversation_id)
        .bind(new_scope.starts_at)
        .bind(new_scope.ends_at)
        .bind(new_scope.purpose)
        .bind(new_scope.expires_at)
        .fetch_one(connection)
        .await
        .map(AnalysisScope::from)
    }
}

#[derive(sqlx::FromRow)]
struct AnalysisScopeRow {
    id: Uuid,
    conversation_id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    purpose: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl From<AnalysisScopeRow> for AnalysisScope {
    fn from(row: AnalysisScopeRow) -> Self {
        Self {
            id: row.id,
            conversation_id: row.conversation_id,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            purpose: row.purpose,
            created_at: row.created_at,
            expires_at: row.expires_at,
            revoked_at: row.revoked_at,
        }
    }
}
