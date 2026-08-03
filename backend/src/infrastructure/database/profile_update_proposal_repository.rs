use crate::domain::profile_update_proposal::{
    ProfileUpdateProposal, ProfileUpdateStatus, ProfileUpdateTarget,
};
use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use std::io::{Error as IoError, ErrorKind};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewProfileUpdateProposal<'a> {
    pub target: ProfileUpdateTarget,
    pub base_sha256: &'a str,
    pub proposed_content: &'a str,
    pub reason: &'a str,
    pub proposed_by: &'a str,
}

pub struct ProfileUpdateProposalRepository;

impl ProfileUpdateProposalRepository {
    pub async fn create(
        connection: &mut PgConnection,
        new_proposal: NewProfileUpdateProposal<'_>,
    ) -> Result<ProfileUpdateProposal, sqlx::Error> {
        let row = sqlx::query_as::<_, ProfileUpdateProposalRow>(
            r#"
            INSERT INTO profile_update_proposals (
                target,
                base_sha256,
                proposed_content,
                reason,
                proposed_by
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id,
                target,
                base_sha256,
                proposed_content,
                reason,
                proposed_by,
                status,
                created_at,
                decided_at
            "#,
        )
        .bind(new_proposal.target.as_str())
        .bind(new_proposal.base_sha256)
        .bind(new_proposal.proposed_content)
        .bind(new_proposal.reason)
        .bind(new_proposal.proposed_by)
        .fetch_one(connection)
        .await?;

        row.try_into()
    }

    pub async fn find_by_id(
        connection: &mut PgConnection,
        proposal_id: Uuid,
    ) -> Result<Option<ProfileUpdateProposal>, sqlx::Error> {
        sqlx::query_as::<_, ProfileUpdateProposalRow>(
            r#"
            SELECT
                id,
                target,
                base_sha256,
                proposed_content,
                reason,
                proposed_by,
                status,
                created_at,
                decided_at
            FROM profile_update_proposals
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .fetch_optional(connection)
        .await?
        .map(TryInto::try_into)
        .transpose()
    }

    pub async fn list_pending(
        connection: &mut PgConnection,
    ) -> Result<Vec<ProfileUpdateProposal>, sqlx::Error> {
        sqlx::query_as::<_, ProfileUpdateProposalRow>(
            r#"
            SELECT
                id,
                target,
                base_sha256,
                proposed_content,
                reason,
                proposed_by,
                status,
                created_at,
                decided_at
            FROM profile_update_proposals
            WHERE status = 'pending'
            ORDER BY created_at, id
            "#,
        )
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
    }
}

#[derive(sqlx::FromRow)]
struct ProfileUpdateProposalRow {
    id: Uuid,
    target: String,
    base_sha256: String,
    proposed_content: String,
    reason: String,
    proposed_by: String,
    status: String,
    created_at: DateTime<Utc>,
    decided_at: Option<DateTime<Utc>>,
}

impl TryFrom<ProfileUpdateProposalRow> for ProfileUpdateProposal {
    type Error = sqlx::Error;

    fn try_from(row: ProfileUpdateProposalRow) -> Result<Self, Self::Error> {
        let target =
            ProfileUpdateTarget::try_from(row.target.as_str()).map_err(invalid_domain_value)?;
        let status =
            ProfileUpdateStatus::try_from(row.status.as_str()).map_err(invalid_domain_value)?;

        Ok(Self {
            id: row.id,
            target,
            base_sha256: row.base_sha256,
            proposed_content: row.proposed_content,
            reason: row.reason,
            proposed_by: row.proposed_by,
            status,
            created_at: row.created_at,
            decided_at: row.decided_at,
        })
    }
}

fn invalid_domain_value(message: String) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(IoError::new(ErrorKind::InvalidData, message)))
}
