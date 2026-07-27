CREATE TABLE import_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_kind TEXT NOT NULL
        CHECK (source_kind = btrim(source_kind) AND source_kind <> ''),
    original_filename TEXT NOT NULL
        CHECK (original_filename = btrim(original_filename) AND original_filename <> ''),
    content_sha256 TEXT NOT NULL
        CONSTRAINT import_batches_content_sha256_unique UNIQUE
        CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    imported_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
