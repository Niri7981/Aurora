use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSenderKind {
    SelfUser,
    Participant,
}

impl MessageSenderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfUser => "self",
            Self::Participant => "participant",
        }
    }
}

impl TryFrom<&str> for MessageSenderKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "self" => Ok(Self::SelfUser),
            "participant" => Ok(Self::Participant),
            _ => Err(format!("unsupported message sender kind: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub source_sequence: i64,
    pub sender_kind: MessageSenderKind,
    pub sender_key: String,
    pub sender_display_name: String,
    pub sent_at: DateTime<Utc>,
    pub content_text: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::MessageSenderKind;

    #[test]
    fn sender_kind_uses_stable_storage_values() {
        assert_eq!(MessageSenderKind::SelfUser.as_str(), "self");
        assert_eq!(MessageSenderKind::Participant.as_str(), "participant");
        assert_eq!(
            MessageSenderKind::try_from("self"),
            Ok(MessageSenderKind::SelfUser)
        );
        assert_eq!(
            MessageSenderKind::try_from("participant"),
            Ok(MessageSenderKind::Participant)
        );
        assert!(MessageSenderKind::try_from("unknown").is_err());
    }
}
