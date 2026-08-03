use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileUpdateTarget {
    IdentityCard,
    CurrentFocus,
    Preferences,
}

impl ProfileUpdateTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IdentityCard => "identity_card",
            Self::CurrentFocus => "current_focus",
            Self::Preferences => "preferences",
        }
    }
}

impl TryFrom<&str> for ProfileUpdateTarget {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "identity_card" => Ok(Self::IdentityCard),
            "current_focus" => Ok(Self::CurrentFocus),
            "preferences" => Ok(Self::Preferences),
            _ => Err(format!("unsupported profile update target: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileUpdateStatus {
    Pending,
    Applied,
    Rejected,
    Stale,
}

impl ProfileUpdateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Applied => "applied",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
        }
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        self == Self::Pending && matches!(next, Self::Applied | Self::Rejected | Self::Stale)
    }
}

impl TryFrom<&str> for ProfileUpdateStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(Self::Pending),
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            "stale" => Ok(Self::Stale),
            _ => Err(format!("unsupported profile update status: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileUpdateProposal {
    pub id: Uuid,
    pub target: ProfileUpdateTarget,
    pub base_sha256: String,
    pub proposed_content: String,
    pub reason: String,
    pub proposed_by: String,
    pub status: ProfileUpdateStatus,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::{ProfileUpdateStatus, ProfileUpdateTarget};

    #[test]
    fn update_targets_use_stable_storage_values_and_exclude_privacy_rules() {
        assert_eq!(ProfileUpdateTarget::IdentityCard.as_str(), "identity_card");
        assert_eq!(ProfileUpdateTarget::CurrentFocus.as_str(), "current_focus");
        assert_eq!(ProfileUpdateTarget::Preferences.as_str(), "preferences");
        assert_eq!(
            ProfileUpdateTarget::try_from("current_focus"),
            Ok(ProfileUpdateTarget::CurrentFocus)
        );
        assert!(ProfileUpdateTarget::try_from("privacy_rules").is_err());
    }

    #[test]
    fn only_pending_proposals_can_reach_a_terminal_status() {
        assert!(ProfileUpdateStatus::Pending.can_transition_to(ProfileUpdateStatus::Applied));
        assert!(ProfileUpdateStatus::Pending.can_transition_to(ProfileUpdateStatus::Rejected));
        assert!(ProfileUpdateStatus::Pending.can_transition_to(ProfileUpdateStatus::Stale));
        assert!(!ProfileUpdateStatus::Pending.can_transition_to(ProfileUpdateStatus::Pending));
        assert!(!ProfileUpdateStatus::Applied.can_transition_to(ProfileUpdateStatus::Rejected));
        assert!(!ProfileUpdateStatus::Rejected.can_transition_to(ProfileUpdateStatus::Applied));
        assert!(!ProfileUpdateStatus::Stale.can_transition_to(ProfileUpdateStatus::Applied));
    }

    #[test]
    fn update_statuses_use_stable_storage_values() {
        for (stored, status) in [
            ("pending", ProfileUpdateStatus::Pending),
            ("applied", ProfileUpdateStatus::Applied),
            ("rejected", ProfileUpdateStatus::Rejected),
            ("stale", ProfileUpdateStatus::Stale),
        ] {
            assert_eq!(status.as_str(), stored);
            assert_eq!(ProfileUpdateStatus::try_from(stored), Ok(status));
        }

        assert!(ProfileUpdateStatus::try_from("approved").is_err());
    }
}
