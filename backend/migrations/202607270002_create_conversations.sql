CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_batch_id UUID NOT NULL
        REFERENCES import_batches(id) ON DELETE RESTRICT,
    source_conversation_key TEXT NOT NULL
        CHECK (
            source_conversation_key = btrim(source_conversation_key)
            AND source_conversation_key <> ''
        ),
    title TEXT NOT NULL
        CHECK (title = btrim(title) AND title <> ''),
    conversation_kind TEXT NOT NULL
        CHECK (conversation_kind IN ('direct', 'group')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT conversations_batch_source_key_unique
        UNIQUE (import_batch_id, source_conversation_key)
);
