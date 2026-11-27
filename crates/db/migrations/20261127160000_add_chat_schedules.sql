-- Messages sent to a group on a schedule, so agents can work unattended.
CREATE TABLE IF NOT EXISTS chat_schedules (
    id               BLOB PRIMARY KEY,
    session_id       BLOB NOT NULL,
    title            TEXT NOT NULL,
    prompt           TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('cron', 'every', 'once')),
    cron_expr        TEXT,
    every_seconds    INTEGER CHECK (every_seconds IS NULL OR every_seconds >= 60),
    run_at           TEXT,
    misfire          TEXT NOT NULL DEFAULT 'run_once' CHECK (misfire IN ('skip', 'run_once')),
    enabled          INTEGER NOT NULL DEFAULT 1,
    next_run_at      TEXT,
    last_run_at      TEXT,
    last_status      TEXT,
    last_message_id  BLOB,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_chat_schedules_due
    ON chat_schedules(enabled, next_run_at);
CREATE INDEX IF NOT EXISTS idx_chat_schedules_session
    ON chat_schedules(session_id);
