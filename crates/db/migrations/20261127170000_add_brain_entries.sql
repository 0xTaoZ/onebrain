-- The team's shared brain: decisions, rules, preferences and facts that agents
-- propose and the user approves. Entries are shared by every group working in
-- the same project folder (scope = folder path; '' = all groups).
CREATE TABLE IF NOT EXISTS brain_entries (
    id                 BLOB NOT NULL UNIQUE,
    scope              TEXT NOT NULL DEFAULT '',
    kind               TEXT NOT NULL CHECK (kind IN ('decision', 'rule', 'preference', 'fact')),
    title              TEXT NOT NULL,
    body               TEXT NOT NULL,
    status             TEXT NOT NULL CHECK (status IN ('proposed', 'active', 'rejected', 'superseded')),
    locked             INTEGER NOT NULL DEFAULT 0,
    source_session_id  BLOB,
    source_run_id      BLOB,
    source_agent       TEXT,
    supersedes_id      BLOB,
    use_count          INTEGER NOT NULL DEFAULT 0,
    last_used_at       TEXT,
    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_brain_entries_scope_status ON brain_entries(scope, status);

-- Trigram full-text index: works for Chinese, German and code identifiers
-- alike, without a word segmenter or an embedding model.
CREATE VIRTUAL TABLE IF NOT EXISTS brain_fts USING fts5(
    title, body, content='brain_entries', content_rowid='rowid', tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS brain_entries_ai AFTER INSERT ON brain_entries BEGIN
    INSERT INTO brain_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
END;
CREATE TRIGGER IF NOT EXISTS brain_entries_ad AFTER DELETE ON brain_entries BEGIN
    INSERT INTO brain_fts(brain_fts, rowid, title, body) VALUES ('delete', old.rowid, old.title, old.body);
END;
CREATE TRIGGER IF NOT EXISTS brain_entries_au AFTER UPDATE OF title, body ON brain_entries BEGIN
    INSERT INTO brain_fts(brain_fts, rowid, title, body) VALUES ('delete', old.rowid, old.title, old.body);
    INSERT INTO brain_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
END;

-- Which entries an agent's current CLI session has already been given, so
-- the same memory is never sent twice to one conversation.
CREATE TABLE IF NOT EXISTS brain_deliveries (
    session_agent_id  BLOB NOT NULL,
    agent_session_id  TEXT NOT NULL,
    entry_id          BLOB NOT NULL,
    PRIMARY KEY (session_agent_id, agent_session_id, entry_id)
);
