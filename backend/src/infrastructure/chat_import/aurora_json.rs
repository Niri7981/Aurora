use crate::domain::conversation::ConversationKind;
use crate::domain::message::MessageSenderKind;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub const AURORA_CHAT_FORMAT_V1: &str = "aurora.chat.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardChatDocument {
    pub format: String,
    pub source: String,
    pub conversations: Vec<StandardConversation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardConversation {
    pub key: String,
    pub title: String,
    pub kind: ConversationKind,
    pub messages: Vec<StandardMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardMessage {
    pub sequence: i64,
    pub sender: StandardSender,
    pub sent_at: DateTime<Utc>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardSender {
    pub kind: MessageSenderKind,
    pub key: String,
    pub display_name: String,
}

#[derive(Debug)]
pub enum AuroraChatJsonError {
    InvalidJson(serde_json::Error),
    InvalidField { path: String, message: String },
}

impl AuroraChatJsonError {
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::InvalidJson(_) => None,
            Self::InvalidField { path, .. } => Some(path),
        }
    }
}

impl Display for AuroraChatJsonError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid Aurora chat JSON: {error}"),
            Self::InvalidField { path, message } => {
                write!(formatter, "invalid {path}: {message}")
            }
        }
    }
}

impl Error for AuroraChatJsonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidJson(error) => Some(error),
            Self::InvalidField { .. } => None,
        }
    }
}

pub fn parse_and_validate(input: &str) -> Result<StandardChatDocument, AuroraChatJsonError> {
    let raw: RawDocument = serde_json::from_str(input).map_err(AuroraChatJsonError::InvalidJson)?;
    validate_document(raw)
}

fn validate_document(raw: RawDocument) -> Result<StandardChatDocument, AuroraChatJsonError> {
    if raw.format != AURORA_CHAT_FORMAT_V1 {
        return Err(invalid_field(
            "format",
            format!("must equal {AURORA_CHAT_FORMAT_V1}"),
        ));
    }
    require_trimmed(&raw.source, "source")?;
    if raw.conversations.is_empty() {
        return Err(invalid_field(
            "conversations",
            "must contain at least one conversation",
        ));
    }

    let mut conversation_keys = HashSet::new();
    let mut conversations = Vec::with_capacity(raw.conversations.len());
    for (index, conversation) in raw.conversations.into_iter().enumerate() {
        let path = format!("conversations[{index}]");
        require_trimmed(&conversation.key, &format!("{path}.key"))?;
        if !conversation_keys.insert(conversation.key.clone()) {
            return Err(invalid_field(
                format!("{path}.key"),
                "must be unique within the file",
            ));
        }
        require_trimmed(&conversation.title, &format!("{path}.title"))?;
        let kind = ConversationKind::try_from(conversation.kind.as_str())
            .map_err(|message| invalid_field(format!("{path}.kind"), message))?;
        if conversation.messages.is_empty() {
            return Err(invalid_field(
                format!("{path}.messages"),
                "must contain at least one message",
            ));
        }

        let mut messages = Vec::with_capacity(conversation.messages.len());
        for (message_index, message) in conversation.messages.into_iter().enumerate() {
            messages.push(validate_message(message, index, message_index)?);
        }

        conversations.push(StandardConversation {
            key: conversation.key,
            title: conversation.title,
            kind,
            messages,
        });
    }

    Ok(StandardChatDocument {
        format: raw.format,
        source: raw.source,
        conversations,
    })
}

fn validate_message(
    raw: RawMessage,
    conversation_index: usize,
    message_index: usize,
) -> Result<StandardMessage, AuroraChatJsonError> {
    let path = format!("conversations[{conversation_index}].messages[{message_index}]");
    let expected_sequence = message_index as i64;
    if raw.sequence != expected_sequence {
        return Err(invalid_field(
            format!("{path}.sequence"),
            format!("must equal {expected_sequence} to match array order"),
        ));
    }

    let sender_path = format!("{path}.sender");
    require_trimmed(&raw.sender.key, &format!("{sender_path}.key"))?;
    require_trimmed(
        &raw.sender.display_name,
        &format!("{sender_path}.display_name"),
    )?;
    let sender_kind = MessageSenderKind::try_from(raw.sender.kind.as_str())
        .map_err(|message| invalid_field(format!("{sender_path}.kind"), message))?;
    match sender_kind {
        MessageSenderKind::SelfUser if raw.sender.key != "self" => {
            return Err(invalid_field(
                format!("{sender_path}.key"),
                "must equal self when sender.kind is self",
            ));
        }
        MessageSenderKind::Participant if raw.sender.key == "self" => {
            return Err(invalid_field(
                format!("{sender_path}.key"),
                "must not equal self when sender.kind is participant",
            ));
        }
        _ => {}
    }

    let sent_at = DateTime::parse_from_rfc3339(&raw.sent_at)
        .map_err(|error| invalid_field(format!("{path}.sent_at"), error.to_string()))?
        .with_timezone(&Utc);
    if raw.text.trim().is_empty() {
        return Err(invalid_field(
            format!("{path}.text"),
            "must contain non-whitespace text",
        ));
    }

    Ok(StandardMessage {
        sequence: raw.sequence,
        sender: StandardSender {
            kind: sender_kind,
            key: raw.sender.key,
            display_name: raw.sender.display_name,
        },
        sent_at,
        text: raw.text,
    })
}

fn require_trimmed(value: &str, path: &str) -> Result<(), AuroraChatJsonError> {
    if value.is_empty() {
        return Err(invalid_field(path, "must not be empty"));
    }
    if value.trim() != value {
        return Err(invalid_field(
            path,
            "must not have leading or trailing whitespace",
        ));
    }
    Ok(())
}

fn invalid_field(path: impl Into<String>, message: impl Into<String>) -> AuroraChatJsonError {
    AuroraChatJsonError::InvalidField {
        path: path.into(),
        message: message.into(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDocument {
    format: String,
    source: String,
    conversations: Vec<RawConversation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConversation {
    key: String,
    title: String,
    kind: String,
    messages: Vec<RawMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMessage {
    sequence: i64,
    sender: RawSender,
    sent_at: String,
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSender {
    kind: String,
    key: String,
    display_name: String,
}
