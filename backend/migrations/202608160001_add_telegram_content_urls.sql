ALTER TABLE telegram_messages
    ADD COLUMN content_urls JSONB NOT NULL DEFAULT '[]'::jsonb
    CONSTRAINT telegram_messages_content_urls_array_check
    CHECK (jsonb_typeof(content_urls) = 'array');

UPDATE telegram_messages
SET content_urls = jsonb_build_array(
    jsonb_build_object(
        'url', external_url,
        'kind', 'legacy_external'
    )
)
WHERE external_url IS NOT NULL;
