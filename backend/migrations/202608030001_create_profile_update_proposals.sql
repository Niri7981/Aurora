CREATE TABLE profile_update_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target TEXT NOT NULL
        CONSTRAINT profile_update_proposals_target_check
        CHECK (target IN ('identity_card', 'current_focus', 'preferences')),
    base_sha256 TEXT NOT NULL
        CONSTRAINT profile_update_proposals_base_sha256_check
        CHECK (base_sha256 ~ '^[0-9a-f]{64}$'),
    proposed_content TEXT NOT NULL
        CONSTRAINT profile_update_proposals_content_check
        CHECK (btrim(proposed_content) <> ''),
    reason TEXT NOT NULL
        CONSTRAINT profile_update_proposals_reason_check
        CHECK (reason = btrim(reason) AND reason <> ''),
    proposed_by TEXT NOT NULL
        CONSTRAINT profile_update_proposals_proposed_by_check
        CHECK (proposed_by = btrim(proposed_by) AND proposed_by <> ''),
    status TEXT NOT NULL DEFAULT 'pending'
        CONSTRAINT profile_update_proposals_status_check
        CHECK (status IN ('pending', 'applied', 'rejected', 'stale')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    decided_at TIMESTAMPTZ,
    CONSTRAINT profile_update_proposals_decision_state_check
        CHECK (
            (status = 'pending' AND decided_at IS NULL)
            OR (status IN ('applied', 'rejected', 'stale') AND decided_at IS NOT NULL)
        ),
    CONSTRAINT profile_update_proposals_decided_at_check
        CHECK (decided_at IS NULL OR created_at <= decided_at)
);
