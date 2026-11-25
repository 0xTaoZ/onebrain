-- What each member's resumed CLI session has already seen.
-- A run stages its cursor when the prompt is built; the cursor is committed
-- only after the run succeeds, so a failed run never hides messages.
CREATE TABLE IF NOT EXISTS chat_member_context_cursors (
    session_agent_id    BLOB PRIMARY KEY,
    agent_session_id    TEXT,
    last_message_id     BLOB,
    prefix_hash         TEXT,
    pending_run_id      BLOB,
    pending_message_id  BLOB,
    pending_prefix_hash TEXT,
    updated_at          TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    FOREIGN KEY (session_agent_id) REFERENCES chat_session_agents(id) ON DELETE CASCADE
);
