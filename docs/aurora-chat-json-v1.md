# Aurora Chat JSON v1

## Purpose

Aurora Chat JSON is the stable, normalized input format for text chat imports. A source-specific adapter may convert WeChat, ChatGPT, Gemini, or another export into this format before Aurora writes anything to PostgreSQL.

Version 1 intentionally supports only text messages. Imported text is evidence and does not automatically become durable personal memory.

## Document Shape

```json
{
  "format": "aurora.chat.v1",
  "source": "wechat",
  "conversations": [
    {
      "key": "wechat:contact:test-friend",
      "title": "我和测试好友",
      "kind": "direct",
      "messages": [
        {
          "sequence": 0,
          "sender": {
            "kind": "self",
            "key": "self",
            "display_name": "我"
          },
          "sent_at": "2025-01-01T10:00:00+08:00",
          "text": "在吗？"
        }
      ]
    }
  ]
}
```

## Database Mapping

| JSON value | PostgreSQL destination |
| --- | --- |
| One input file | One `import_batches` row |
| `source` | `import_batches.source_kind` |
| `conversations[]` | `conversations` rows |
| `conversations[].messages[]` | `messages` rows |

The input does not declare its filename, SHA-256 fingerprint, import time, database IDs, or authorization scopes. Aurora derives those values locally.

## Fields

### Document

| Field | Type | Rule |
| --- | --- | --- |
| `format` | string | Must equal `aurora.chat.v1` |
| `source` | string | Trimmed, non-empty source platform such as `wechat` |
| `conversations` | array | Must contain at least one conversation |

### Conversation

| Field | Type | Rule |
| --- | --- | --- |
| `key` | string | Trimmed, non-empty, and unique within the file |
| `title` | string | Trimmed and non-empty |
| `kind` | string | `direct` or `group` |
| `messages` | array | Must contain at least one message |

### Message

| Field | Type | Rule |
| --- | --- | --- |
| `sequence` | integer | Starts at `0` and increases by `1` without gaps inside one conversation |
| `sender.kind` | string | `self` or `participant` |
| `sender.key` | string | Stable, trimmed, non-empty sender key |
| `sender.display_name` | string | Trimmed, non-empty name captured at import time |
| `sent_at` | string | RFC 3339 timestamp with an explicit UTC offset |
| `text` | string | Must contain non-whitespace text; its original leading and trailing whitespace is preserved |

When `sender.kind` is `self`, `sender.key` must be `self`. A participant key must not be `self`.

## Ordering And Time

Array order and `sequence` must agree. `sequence` is the normalized position in this input file, not a provider-specific message ID. Timestamps are stored as PostgreSQL `TIMESTAMPTZ`; the represented instant is preserved even when the input uses a non-UTC offset.

## Compatibility Rules

- The v1 parser will reject unknown `format` values and unknown fields instead of silently ignoring a typo.
- A future incompatible shape will use a new format value such as `aurora.chat.v2`.
- Images, voice messages, video, files, reactions, edits, recalls, and replies are outside v1.
- File hashing and duplicate-import detection happen before parsing and are not trusted to input fields.

The repository includes a synthetic fixture at `backend/tests/fixtures/aurora-chat-v1.json`. It contains no real personal information.
