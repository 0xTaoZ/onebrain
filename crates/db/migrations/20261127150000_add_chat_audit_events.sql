-- What agents asked to do and what was decided, kept for 90 days.
CREATE TABLE IF NOT EXISTS chat_audit_events (
    id                BLOB PRIMARY KEY,
    session_id        BLOB NOT NULL,
    session_agent_id  BLOB,
    run_id            BLOB,
    kind              TEXT NOT NULL,
    dangerous         INTEGER NOT NULL DEFAULT 0,
    summary           TEXT NOT NULL,
    detail            TEXT NOT NULL DEFAULT '{}',
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_chat_audit_events_session_time
    ON chat_audit_events(session_id, created_at DESC, id DESC);
