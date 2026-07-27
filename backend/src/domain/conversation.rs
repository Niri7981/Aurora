use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationKind {
    Direct,
    Group,
}

impl ConversationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Group => "group",
        }
    }
}

impl TryFrom<&str> for ConversationKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "direct" => Ok(Self::Direct),
            "group" => Ok(Self::Group),
            _ => Err(format!("unsupported conversation kind: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub id: Uuid,
    pub import_batch_id: Uuid,
    pub source_conversation_key: String,
    pub title: String,
    pub kind: ConversationKind,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::ConversationKind;

    #[test]
    fn conversation_kind_uses_stable_storage_values() {
        assert_eq!(ConversationKind::Direct.as_str(), "direct");
        assert_eq!(ConversationKind::Group.as_str(), "group");
        assert_eq!(
            ConversationKind::try_from("direct"),
            Ok(ConversationKind::Direct)
        );
        assert_eq!(
            ConversationKind::try_from("group"),
            Ok(ConversationKind::Group)
        );
        assert!(ConversationKind::try_from("channel").is_err());
    }
}
