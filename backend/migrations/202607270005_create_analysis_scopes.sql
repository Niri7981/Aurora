CREATE TABLE analysis_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL
        REFERENCES conversations(id) ON DELETE RESTRICT,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    purpose TEXT NOT NULL
        CHECK (purpose = btrim(purpose) AND purpose <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT analysis_scopes_time_range_check
        CHECK (starts_at < ends_at),
    CONSTRAINT analysis_scopes_expiration_check
        CHECK (created_at < expires_at),
    CONSTRAINT analysis_scopes_revocation_check
        CHECK (revoked_at IS NULL OR created_at <= revoked_at)
);
