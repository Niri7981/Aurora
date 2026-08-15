CREATE TABLE telegram_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_name TEXT NOT NULL
        CONSTRAINT telegram_messages_channel_name_check
        CHECK (channel_name = btrim(channel_name) AND channel_name <> '' AND char_length(channel_name) <= 500),
    content_text TEXT NOT NULL
        CONSTRAINT telegram_messages_content_text_check
        CHECK (btrim(content_text) <> '' AND char_length(content_text) <= 50000),
    author_name TEXT
        CONSTRAINT telegram_messages_author_name_check
        CHECK (author_name IS NULL OR (author_name = btrim(author_name) AND author_name <> '' AND char_length(author_name) <= 500)),
    published_at TIMESTAMPTZ,
    external_message_id TEXT
        CONSTRAINT telegram_messages_external_message_id_check
        CHECK (external_message_id IS NULL OR (external_message_id = btrim(external_message_id) AND external_message_id <> '' AND char_length(external_message_id) <= 500)),
    external_url TEXT
        CONSTRAINT telegram_messages_external_url_check
        CHECK (external_url IS NULL OR (external_url = btrim(external_url) AND external_url <> '' AND char_length(external_url) <= 2048)),
    dedup_sha256 TEXT NOT NULL UNIQUE
        CONSTRAINT telegram_messages_dedup_sha256_check
        CHECK (dedup_sha256 ~ '^[0-9a-f]{64}$'),
    saved_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX telegram_messages_channel_published_at_idx
    ON telegram_messages (channel_name, published_at DESC NULLS LAST, saved_at DESC);
