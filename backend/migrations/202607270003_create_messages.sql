CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL
        REFERENCES conversations(id) ON DELETE RESTRICT,
    source_sequence BIGINT NOT NULL
        CHECK (source_sequence >= 0),
    sender_key TEXT NOT NULL
        CHECK (sender_key = btrim(sender_key) AND sender_key <> ''),
    sender_display_name TEXT NOT NULL
        CHECK (
            sender_display_name = btrim(sender_display_name)
            AND sender_display_name <> ''
        ),
    sent_at TIMESTAMPTZ NOT NULL,
    content_text TEXT NOT NULL
        CHECK (btrim(content_text) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT messages_conversation_source_sequence_unique
        UNIQUE (conversation_id, source_sequence)
);
