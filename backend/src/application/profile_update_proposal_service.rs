use std::fs;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::profile_update_proposal::{ProfileUpdateProposal, ProfileUpdateTarget};
use crate::infrastructure::database::profile_update_proposal_repository::{
    NewProfileUpdateProposal, ProfileUpdateProposalRepository,
};

#[derive(Clone)]
pub struct ProfileUpdateProposalService {
    config: AppConfig,
    client: String,
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
    Ok(format!("{:x}", Sha256::digest(content)))
}

fn required_text<'a>(value: &'a str, field: &str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}
