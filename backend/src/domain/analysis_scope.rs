use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisScope {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub purpose: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl AnalysisScope {
    pub fn contains_message_at(&self, sent_at: DateTime<Utc>) -> bool {
        self.starts_at <= sent_at && sent_at < self.ends_at
    }

    pub fn is_active_at(&self, checked_at: DateTime<Utc>) -> bool {
        let before_revocation = match self.revoked_at {
            Some(revoked_at) => checked_at < revoked_at,
            None => true,
        };

        self.created_at <= checked_at && checked_at < self.expires_at && before_revocation
    }
}

#[cfg(test)]
mod tests {
    use super::AnalysisScope;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn timestamp(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("test timestamp should be valid")
            .with_timezone(&Utc)
    }

    fn scope() -> AnalysisScope {
        AnalysisScope {
            id: Uuid::nil(),
            conversation_id: Uuid::nil(),
            starts_at: timestamp("2025-01-01T00:00:00Z"),
            ends_at: timestamp("2025-02-01T00:00:00Z"),
            purpose: "分析关系变化".to_string(),
            created_at: timestamp("2026-07-27T00:00:00Z"),
            expires_at: timestamp("2026-07-28T00:00:00Z"),
            revoked_at: None,
        }
    }

    #[test]
    fn message_range_includes_start_and_excludes_end() {
        let scope = scope();

        assert!(scope.contains_message_at(scope.starts_at));
        assert!(scope.contains_message_at(timestamp("2025-01-31T23:59:59Z")));
        assert!(!scope.contains_message_at(scope.ends_at));
    }

    #[test]
    fn active_scope_honors_expiration_and_revocation() {
        let mut scope = scope();

        assert!(scope.is_active_at(timestamp("2026-07-27T12:00:00Z")));
        assert!(!scope.is_active_at(scope.expires_at));

        scope.revoked_at = Some(timestamp("2026-07-27T13:00:00Z"));
        assert!(scope.is_active_at(timestamp("2026-07-27T12:59:59Z")));
        assert!(!scope.is_active_at(timestamp("2026-07-27T13:00:00Z")));
    }
}
