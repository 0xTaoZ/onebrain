ALTER TABLE chat_messages ADD COLUMN client_op_key TEXT;
CREATE UNIQUE INDEX chat_messages_session_op_key_unique
ON chat_messages(session_id, client_op_key) WHERE client_op_key IS NOT NULL;
