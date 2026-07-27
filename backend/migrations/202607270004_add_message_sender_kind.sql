ALTER TABLE messages
    ADD COLUMN sender_kind TEXT;

UPDATE messages
SET sender_kind = CASE
    WHEN sender_key = 'self' THEN 'self'
    ELSE 'participant'
END;

ALTER TABLE messages
    ALTER COLUMN sender_kind SET NOT NULL,
    ADD CONSTRAINT messages_sender_kind_check
        CHECK (sender_kind IN ('self', 'participant'));
