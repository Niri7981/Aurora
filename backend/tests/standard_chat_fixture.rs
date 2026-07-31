use chrono::DateTime;
use serde_json::Value;
use std::collections::HashSet;

#[test]
fn synthetic_fixture_matches_the_documented_v1_shape() {
    let document: Value = serde_json::from_str(include_str!("fixtures/aurora-chat-v1.json"))
        .expect("standard chat fixture should be valid JSON");

    assert_eq!(document["format"], "aurora.chat.v1");
    assert_eq!(document["source"], "wechat");

    let conversations = document["conversations"]
        .as_array()
        .expect("fixture should contain conversations");
    assert_eq!(conversations.len(), 2);
    assert_eq!(conversations[0]["kind"], "direct");
    assert_eq!(conversations[1]["kind"], "group");

    let mut conversation_keys = HashSet::new();
    for conversation in conversations {
        let key = required_string(conversation, "key");
        assert!(conversation_keys.insert(key));
        assert!(!required_string(conversation, "title").trim().is_empty());

        let messages = conversation["messages"]
            .as_array()
            .expect("fixture conversation should contain messages");
        assert!(!messages.is_empty());

        for (expected_sequence, message) in messages.iter().enumerate() {
            assert_eq!(message["sequence"].as_u64(), Some(expected_sequence as u64));

            let sender_kind = required_string(&message["sender"], "kind");
            let sender_key = required_string(&message["sender"], "key");
            match sender_kind {
                "self" => assert_eq!(sender_key, "self"),
                "participant" => assert_ne!(sender_key, "self"),
                value => panic!("unexpected sender kind: {value}"),
            }

            assert!(
                !required_string(&message["sender"], "display_name")
                    .trim()
                    .is_empty()
            );
            DateTime::parse_from_rfc3339(required_string(message, "sent_at"))
                .expect("fixture timestamp should be RFC 3339 with an offset");
            assert!(!required_string(message, "text").trim().is_empty());
        }
    }
}

fn required_string<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("fixture field {field} should be a string"))
}
