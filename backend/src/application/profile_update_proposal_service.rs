use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::profile_update_proposal::{
    ProfileUpdateProposal, ProfileUpdateStatus, ProfileUpdateTarget,
};
use crate::infrastructure::database::profile_update_proposal_repository::{
    NewProfileUpdateProposal, ProfileUpdateProposalRepository,
};

#[derive(Clone)]
pub struct ProfileUpdateProposalService {
    config: AppConfig,
    client: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyProfileUpdateOutcome {
    Applied(ProfileUpdateProposal),
    Stale(ProfileUpdateProposal),
    NotPending(ProfileUpdateProposal),
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteProfileUpdateOutcome {
    Deleted(ProfileUpdateProposal),
    NotPending(ProfileUpdateProposal),
    NotFound,
}

impl ProfileUpdateProposalService {
    pub fn new(config: AppConfig, client: impl Into<String>) -> Self {
        Self {
            config,
            client: client.into(),
        }
    }

    pub async fn propose(
        &self,
        connection: &mut PgConnection,
        target: ProfileUpdateTarget,
        proposed_content: &str,
        reason: &str,
    ) -> Result<ProfileUpdateProposal, String> {
        validate_proposed_content(target, proposed_content)?;
        let reason = required_text(reason, "reason")?;
        let proposed_by = required_text(&self.client, "MCP client identity")?;
        let base_sha256 = sha256_file(self.target_path(target))?;

        ProfileUpdateProposalRepository::create(
            connection,
            NewProfileUpdateProposal {
                target,
                base_sha256: &base_sha256,
                proposed_content,
                reason,
                proposed_by,
            },
        )
        .await
        .map_err(|error| format!("failed to create profile update proposal: {error}"))
    }

    pub async fn find_by_id(
        &self,
        connection: &mut PgConnection,
        proposal_id: Uuid,
    ) -> Result<Option<ProfileUpdateProposal>, String> {
        ProfileUpdateProposalRepository::find_by_id(connection, proposal_id)
            .await
            .map_err(|error| format!("failed to read profile update proposal: {error}"))
    }

    pub async fn list_pending(
        &self,
        connection: &mut PgConnection,
    ) -> Result<Vec<ProfileUpdateProposal>, String> {
        ProfileUpdateProposalRepository::list_pending(connection)
            .await
            .map_err(|error| format!("failed to list profile update proposals: {error}"))
    }

    pub async fn apply(
        &self,
        pool: &PgPool,
        proposal_id: Uuid,
    ) -> Result<ApplyProfileUpdateOutcome, String> {
        let mut transaction = pool
            .begin()
            .await
            .map_err(|error| format!("failed to start profile update transaction: {error}"))?;
        let proposal = match ProfileUpdateProposalRepository::find_by_id_for_update(
            &mut transaction,
            proposal_id,
        )
        .await
        .map_err(|error| format!("failed to lock profile update proposal: {error}"))?
        {
            Some(proposal) => proposal,
            None => {
                transaction.rollback().await.map_err(|error| {
                    format!("failed to close missing proposal transaction: {error}")
                })?;
                return Ok(ApplyProfileUpdateOutcome::NotFound);
            }
        };

        if proposal.status != ProfileUpdateStatus::Pending {
            transaction.rollback().await.map_err(|error| {
                format!("failed to close decided proposal transaction: {error}")
            })?;
            return Ok(ApplyProfileUpdateOutcome::NotPending(proposal));
        }

        ProfileUpdateProposalRepository::lock_target(&mut transaction, proposal.target)
            .await
            .map_err(|error| format!("failed to lock profile update target: {error}"))?;

        let path = self.target_path(proposal.target);
        let original_content = fs::read(path).map_err(|error| {
            format!(
                "failed to read current profile at {}: {error}",
                path.display()
            )
        })?;
        let current_sha256 = sha256_bytes(&original_content);
        if current_sha256 != proposal.base_sha256 {
            let stale = ProfileUpdateProposalRepository::transition_pending(
                &mut transaction,
                proposal.id,
                ProfileUpdateStatus::Stale,
            )
            .await
            .map_err(|error| format!("failed to mark profile update proposal stale: {error}"))?
            .ok_or_else(|| "pending profile update proposal changed unexpectedly".to_string())?;
            transaction
                .commit()
                .await
                .map_err(|error| format!("failed to commit stale proposal status: {error}"))?;
            return Ok(ApplyProfileUpdateOutcome::Stale(stale));
        }

        validate_proposed_content(proposal.target, &proposal.proposed_content)?;
        atomic_replace(path, proposal.proposed_content.as_bytes(), proposal.id)?;

        let applied = match ProfileUpdateProposalRepository::transition_pending(
            &mut transaction,
            proposal.id,
            ProfileUpdateStatus::Applied,
        )
        .await
        {
            Ok(Some(applied)) => applied,
            Ok(None) => {
                restore_after_failed_apply(path, &original_content, proposal.id)?;
                return Err("pending profile update proposal changed unexpectedly".to_string());
            }
            Err(error) => {
                restore_after_failed_apply(path, &original_content, proposal.id)?;
                return Err(format!(
                    "failed to mark profile update proposal applied: {error}"
                ));
            }
        };

        if let Err(error) = transaction.commit().await {
            restore_after_failed_apply(path, &original_content, proposal.id)?;
            return Err(format!("failed to commit applied proposal status: {error}"));
        }

        Ok(ApplyProfileUpdateOutcome::Applied(applied))
    }

    pub async fn delete_pending(
        &self,
        pool: &PgPool,
        proposal_id: Uuid,
    ) -> Result<DeleteProfileUpdateOutcome, String> {
        let mut transaction = pool
            .begin()
            .await
            .map_err(|error| format!("failed to start proposal deletion transaction: {error}"))?;
        let proposal = match ProfileUpdateProposalRepository::find_by_id_for_update(
            &mut transaction,
            proposal_id,
        )
        .await
        .map_err(|error| format!("failed to lock profile update proposal: {error}"))?
        {
            Some(proposal) => proposal,
            None => {
                transaction.rollback().await.map_err(|error| {
                    format!("failed to close missing proposal transaction: {error}")
                })?;
                return Ok(DeleteProfileUpdateOutcome::NotFound);
            }
        };

        if proposal.status != ProfileUpdateStatus::Pending {
            transaction.rollback().await.map_err(|error| {
                format!("failed to close retained proposal transaction: {error}")
            })?;
            return Ok(DeleteProfileUpdateOutcome::NotPending(proposal));
        }

        let deleted =
            ProfileUpdateProposalRepository::delete_pending(&mut transaction, proposal.id)
                .await
                .map_err(|error| {
                    format!("failed to delete pending profile update proposal: {error}")
                })?;
        if !deleted {
            return Err("pending profile update proposal changed unexpectedly".to_string());
        }
        transaction
            .commit()
            .await
            .map_err(|error| format!("failed to commit proposal deletion: {error}"))?;

        Ok(DeleteProfileUpdateOutcome::Deleted(proposal))
    }

    fn target_path(&self, target: ProfileUpdateTarget) -> &Path {
        match target {
            ProfileUpdateTarget::IdentityCard => &self.config.identity_card_path,
            ProfileUpdateTarget::CurrentFocus => &self.config.current_focus_path,
            ProfileUpdateTarget::Preferences => &self.config.preferences_path,
        }
    }
}

fn validate_proposed_content(
    target: ProfileUpdateTarget,
    proposed_content: &str,
) -> Result<(), String> {
    required_text(proposed_content, "proposed_content")?;
    if target == ProfileUpdateTarget::Preferences {
        serde_json::from_str::<Value>(proposed_content)
            .map_err(|error| format!("proposed preferences must be valid JSON: {error}"))?;
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let content = fs::read(path).map_err(|error| {
        format!(
            "failed to read current profile at {}: {error}",
            path.display()
        )
    })?;
    Ok(sha256_bytes(&content))
}

fn sha256_bytes(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn atomic_replace(path: &Path, content: &[u8], proposal_id: Uuid) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("profile path has no parent: {}", path.display()))?;
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("profile path has no valid filename: {}", path.display()))?;
    let temporary_path = parent.join(format!(".{filename}.aurora-{proposal_id}.tmp"));
    let permissions = fs::metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?
        .permissions();

    let result = (|| -> Result<(), String> {
        let mut temporary = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)
            .map_err(|error| {
                format!(
                    "failed to create temporary profile {}: {error}",
                    temporary_path.display()
                )
            })?;
        temporary.write_all(content).map_err(|error| {
            format!(
                "failed to write temporary profile {}: {error}",
                temporary_path.display()
            )
        })?;
        temporary.set_permissions(permissions).map_err(|error| {
            format!(
                "failed to preserve profile permissions on {}: {error}",
                temporary_path.display()
            )
        })?;
        temporary.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary profile {}: {error}",
                temporary_path.display()
            )
        })?;
        fs::rename(&temporary_path, path).map_err(|error| {
            format!(
                "failed to replace profile {} with {}: {error}",
                path.display(),
                temporary_path.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn restore_after_failed_apply(
    path: &Path,
    original_content: &[u8],
    proposal_id: Uuid,
) -> Result<(), String> {
    atomic_replace(path, original_content, proposal_id).map_err(|restore_error| {
        format!(
            "database update failed and the original profile could not be restored: {restore_error}"
        )
    })
}

fn required_text<'a>(value: &'a str, field: &str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}
