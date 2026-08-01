use aurora::domain::conversation::ConversationKind;
use aurora::domain::message::MessageSenderKind;
use aurora::infrastructure::chat_import::aurora_json::{AuroraChatJsonError, parse_and_validate};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/aurora-chat-v1.json");

#[test]
fn parses_the_v1_fixture_into_strong_types_without_rewriting_text() {
    let document = parse_and_validate(FIXTURE).expect("v1 fixture should be valid");

    assert_eq!(document.format, "aurora.chat.v1");
    assert_eq!(document.source, "wechat");
    assert_eq!(document.conversations.len(), 2);
    assert_eq!(document.conversations[0].kind, ConversationKind::Direct);
    assert_eq!(document.conversations[1].kind, ConversationKind::Group);

    let first = &document.conversations[0].messages[0];
    assert_eq!(first.sender.kind, MessageSenderKind::SelfUser);
    assert_eq!(first.text, "  原文首尾空格会保留  ");
    assert_eq!(first.sent_at.to_rfc3339(), "2025-01-01T02:00:00+00:00");
}

#[test]
fn rejects_unknown_fields_during_deserialization() {
    let mut document = fixture_value();
    document["unexpected"] = Value::Bool(true);

    let error = parse_and_validate(&document.to_string()).expect_err("unknown field must fail");
    assert!(matches!(error, AuroraChatJsonError::InvalidJson(_)));
    assert!(error.to_string().contains("unknown field `unexpected`"));
}

#[test]
fn reports_semantic_errors_with_their_document_path() {
    assert_validation_path(
        |document| document["format"] = Value::String("aurora.chat.v2".to_string()),
        "format",
    );
    assert_validation_path(
        |document| {
            document["conversations"][1]["key"] = document["conversations"][0]["key"].clone();
        },
        "conversations[1].key",
    );
    assert_validation_path(
        |document| document["conversations"][0]["messages"][1]["sequence"] = Value::from(7),
        "conversations[0].messages[1].sequence",
    );
    assert_validation_path(
        |document| {
            document["conversations"][0]["messages"][0]["sender"]["key"] =
                Value::String("not-self".to_string());
        },
        "conversations[0].messages[0].sender.key",
    );
    assert_validation_path(
        |document| {
            document["conversations"][0]["messages"][0]["sent_at"] =
                Value::String("2025-01-01 10:00:00".to_string());
        },
        "conversations[0].messages[0].sent_at",
    );
    assert_validation_path(
        |document| {
            document["conversations"][0]["messages"][0]["text"] = Value::String("   ".to_string());
        },
        "conversations[0].messages[0].text",
    );
}

fn fixture_value() -> Value {
    serde_json::from_str(FIXTURE).expect("fixture should be valid JSON")
}

fn assert_validation_path(change: impl FnOnce(&mut Value), expected_path: &str) {
    let mut document = fixture_value();
    change(&mut document);

    let error = parse_and_validate(&document.to_string()).expect_err("invalid field must fail");
    assert_eq!(error.path(), Some(expected_path));
}
