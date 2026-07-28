CREATE INDEX messages_conversation_sent_at_sequence_idx
    ON messages (conversation_id, sent_at, source_sequence);
