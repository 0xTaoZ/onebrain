-- Working-tree snapshot taken before an agent run, so its changes can be undone.
CREATE TABLE IF NOT EXISTS chat_run_checkpoints (
    run_id          BLOB PRIMARY KEY,
    session_id      BLOB NOT NULL,
    workspace_path  TEXT NOT NULL,
    checkpoint      TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
);
