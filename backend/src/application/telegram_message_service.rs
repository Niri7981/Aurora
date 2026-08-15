use crate::infrastructure::database::telegram_message_repository::{
    NewTelegramMessage, SaveTelegramMessageOutcome, TelegramMessageRepository,
};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgConnection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveTelegramMessage<'a> {
    pub channel_name: &'a str,
    pub content_text: &'a str,
    pub author_name: Option<&'a str>,
    pub published_at: Option<DateTime<Utc>>,
    pub external_message_id: Option<&'a str>,
    pub external_url: Option<&'a str>,
}

pub struct TelegramMessageService;

impl TelegramMessageService {
    pub async fn save(
        connection: &mut PgConnection,
        message: SaveTelegramMessage<'_>,
    ) -> Result<SaveTelegramMessageOutcome, String> {
        validate_message(&message)?;
        let dedup_sha256 = dedup_sha256(&message);
        TelegramMessageRepository::save(
            connection,
            NewTelegramMessage {
                channel_name: message.channel_name,
                content_text: message.content_text,
                author_name: message.author_name,
                published_at: message.published_at,
                external_message_id: message.external_message_id,
                external_url: message.external_url,
                dedup_sha256: &dedup_sha256,
            },
        )
        .await
        .map_err(|error| format!("failed to save Telegram message: {error}"))
    }
}

fn validate_message(message: &SaveTelegramMessage<'_>) -> Result<(), String> {
    required_bounded_text(message.channel_name, "channel_name", 500)?;
    required_bounded_text(message.content_text, "content_text", 50_000)?;
    optional_bounded_text(message.author_name, "author_name", 500)?;
    optional_bounded_text(message.external_message_id, "external_message_id", 500)?;
    optional_bounded_text(message.external_url, "external_url", 2_048)?;
    Ok(())
}

fn required_bounded_text(value: &str, field: &str, max_chars: usize) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if value.trim() != value {
        return Err(format!(
            "{field} must not have leading or trailing whitespace"
        ));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{field} must be at most {max_chars} characters"));
    }
    Ok(())
}

fn optional_bounded_text(value: Option<&str>, field: &str, max_chars: usize) -> Result<(), String> {
    match value {
        Some(value) => required_bounded_text(value, field, max_chars),
        None => Ok(()),
    }
}

fn dedup_sha256(message: &SaveTelegramMessage<'_>) -> String {
    let identity = if let Some(url) = message.external_url {
        format!("url\0{url}")
    } else if let Some(message_id) = message.external_message_id {
        format!("message\0{}\0{message_id}", message.channel_name)
    } else {
        format!(
            "content\0{}\0{}\0{}",
            message.channel_name,
            message
                .published_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_default(),
            message.content_text
        )
    };
    format!("{:x}", Sha256::digest(identity.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::{SaveTelegramMessage, dedup_sha256, validate_message};

    #[test]
    fn deduplication_prefers_stable_source_identity() {
        let first = SaveTelegramMessage {
            channel_name: "Example jobs",
            content_text: "First extraction",
            author_name: None,
            published_at: None,
            external_message_id: Some("42"),
            external_url: Some("https://t.me/example/42"),
        };
        let changed_extraction = SaveTelegramMessage {
            content_text: "Improved extraction",
            ..first
        };

        assert_eq!(dedup_sha256(&first), dedup_sha256(&changed_extraction));
    }

    #[test]
    fn validates_required_and_optional_message_fields() {
        let valid = SaveTelegramMessage {
            channel_name: "Example jobs",
            content_text: "A fictional Rust role.",
            author_name: None,
            published_at: None,
            external_message_id: None,
            external_url: None,
        };
        assert!(validate_message(&valid).is_ok());

        assert!(
            validate_message(&SaveTelegramMessage {
                content_text: "   ",
                ..valid
            })
            .unwrap_err()
            .contains("content_text")
        );
        assert!(
            validate_message(&SaveTelegramMessage {
                author_name: Some(" "),
                ..valid
            })
            .unwrap_err()
            .contains("author_name")
        );
    }
}
