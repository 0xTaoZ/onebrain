use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, TS)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(use_ts_enum)]
pub enum BrainEntryKind {
    Decision,
    Rule,
    Preference,
    Fact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, TS)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(use_ts_enum)]
pub enum BrainEntryStatus {
    Proposed,
    Active,
    Rejected,
    Superseded,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct BrainEntry {
    pub id: Uuid,
    pub scope: String,
    pub kind: BrainEntryKind,
    pub title: String,
    pub body: String,
    pub status: BrainEntryStatus,
    pub locked: bool,
    pub source_session_id: Option<Uuid>,
    pub source_agent: Option<String>,
    pub supersedes_id: Option<Uuid>,
    #[ts(type = "number")]
    pub use_count: i64,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

/// A search hit with its full-text rank (lower is better, as in SQLite bm25).
#[derive(Debug, Clone, FromRow)]
pub struct BrainHit {
    #[sqlx(flatten)]
    pub entry: BrainEntry,
    pub rank: f64,
}

#[derive(Debug, Clone)]
pub struct NewBrainEntry {
    pub scope: String,
    pub kind: BrainEntryKind,
    pub title: String,
    pub body: String,
    pub status: BrainEntryStatus,
    pub source_session_id: Option<Uuid>,
    pub source_run_id: Option<Uuid>,
    pub source_agent: Option<String>,
}

const COLUMNS: &str = "e.id, e.scope, e.kind, e.title, e.body, e.status, e.locked, e.source_session_id, \
     e.source_agent, e.supersedes_id, e.use_count, e.last_used_at, e.created_at";

impl BrainEntry {
    /// Add an entry. A proposal whose text already exists in the same scope
    /// (in any state) is ignored, so agents repeating themselves add nothing.
    pub async fn create(
        pool: &SqlitePool,
        entry: &NewBrainEntry,
    ) -> Result<Option<Self>, sqlx::Error> {
        let duplicate: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM brain_entries WHERE scope = ?1 AND body = ?2 LIMIT 1",
        )
        .bind(&entry.scope)
        .bind(&entry.body)
        .fetch_optional(pool)
        .await?;
        if duplicate.is_some() {
            return Ok(None);
        }
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO brain_entries (id, scope, kind, title, body, status, source_session_id, source_run_id, source_agent) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(id)
        .bind(&entry.scope)
        .bind(entry.kind)
        .bind(&entry.title)
        .bind(&entry.body)
        .bind(entry.status)
        .bind(entry.source_session_id)
        .bind(entry.source_run_id)
        .bind(&entry.source_agent)
        .execute(pool)
        .await?;
        Self::find(pool, id).await
    }

    pub async fn find(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT {COLUMNS} FROM brain_entries e WHERE e.id = ?1"
        ))
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Entries for a project folder plus the ones shared by every group.
    pub async fn list(
        pool: &SqlitePool,
        scope: &str,
        status: BrainEntryStatus,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT {COLUMNS} FROM brain_entries e WHERE e.scope IN (?1, '') AND e.status = ?2 \
             ORDER BY e.created_at DESC LIMIT 500"
        ))
        .bind(scope)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    /// Change the text or state. Editing by the user locks the entry so agents
    /// can never overwrite it.
    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        kind: Option<BrainEntryKind>,
        title: Option<&str>,
        body: Option<&str>,
        status: Option<BrainEntryStatus>,
        lock: bool,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query(
            "UPDATE brain_entries SET kind = COALESCE(?2, kind), title = COALESCE(?3, title), \
             body = COALESCE(?4, body), status = COALESCE(?5, status), locked = MAX(locked, ?6), \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        )
        .bind(id)
        .bind(kind)
        .bind(title)
        .bind(body)
        .bind(status)
        .bind(lock)
        .execute(pool)
        .await?;
        Self::find(pool, id).await
    }

    /// Approve `id` as the replacement of `old`: the old entry is kept as
    /// history (superseded) instead of being deleted.
    pub async fn supersede(pool: &SqlitePool, id: Uuid, old: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("UPDATE brain_entries SET status = 'superseded' WHERE id = ?1")
            .bind(old)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE brain_entries SET status = 'active', supersedes_id = ?2 WHERE id = ?1")
            .bind(id)
            .bind(old)
            .execute(&mut *tx)
            .await?;
        tx.commit().await
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM brain_entries WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Active entries matching an FTS5 query, best first.
    pub async fn search(
        pool: &SqlitePool,
        scope: &str,
        fts_query: &str,
        limit: i64,
    ) -> Result<Vec<BrainHit>, sqlx::Error> {
        sqlx::query_as::<_, BrainHit>(&format!(
            "SELECT {COLUMNS}, bm25(brain_fts, 2.0, 1.0) AS rank FROM brain_fts \
             JOIN brain_entries e ON e.rowid = brain_fts.rowid \
             WHERE brain_fts MATCH ?1 AND e.status = 'active' AND e.scope IN (?2, '') \
             ORDER BY rank LIMIT ?3"
        ))
        .bind(fts_query)
        .bind(scope)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// Active rules apply to every task, so they are sent regardless of the topic.
    pub async fn active_rules(
        pool: &SqlitePool,
        scope: &str,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT {COLUMNS} FROM brain_entries e WHERE e.status = 'active' AND e.kind = 'rule' \
             AND e.scope IN (?1, '') ORDER BY e.locked DESC, e.use_count DESC, e.created_at DESC LIMIT ?2"
        ))
        .bind(scope)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn delivered_ids(
        pool: &SqlitePool,
        session_agent_id: Uuid,
        agent_session_id: &str,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT entry_id FROM brain_deliveries WHERE session_agent_id = ?1 AND agent_session_id = ?2",
        )
        .bind(session_agent_id)
        .bind(agent_session_id)
        .fetch_all(pool)
        .await
    }

    /// Remember that these entries were sent, and count the use.
    pub async fn mark_delivered(
        pool: &SqlitePool,
        session_agent_id: Uuid,
        agent_session_id: Option<&str>,
        ids: &[Uuid],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        for id in ids {
            sqlx::query(
                "UPDATE brain_entries SET use_count = use_count + 1, \
                 last_used_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;
            if let Some(agent_session_id) = agent_session_id {
                sqlx::query(
                    "INSERT OR IGNORE INTO brain_deliveries (session_agent_id, agent_session_id, entry_id) VALUES (?1, ?2, ?3)",
                )
                .bind(session_agent_id)
                .bind(agent_session_id)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    async fn pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../../migrations/20261127170000_add_brain_entries.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    fn entry(scope: &str, kind: BrainEntryKind, title: &str, body: &str) -> NewBrainEntry {
        NewBrainEntry {
            scope: scope.into(),
            kind,
            title: title.into(),
            body: body.into(),
            status: BrainEntryStatus::Active,
            source_session_id: None,
            source_run_id: None,
            source_agent: Some("claude".into()),
        }
    }

    #[tokio::test]
    async fn search_finds_chinese_and_english_and_respects_scope_and_status() {
        let pool = pool().await;
        let app = "/work/app";
        BrainEntry::create(
            &pool,
            &entry(
                app,
                BrainEntryKind::Decision,
                "数据库选型",
                "我们用 SQLite 存储所有聊天记录，不用 Postgres",
            ),
        )
        .await
        .unwrap();
        BrainEntry::create(
            &pool,
            &entry(
                app,
                BrainEntryKind::Rule,
                "Commit style",
                "Commit messages use plain engineering language",
            ),
        )
        .await
        .unwrap();
        BrainEntry::create(
            &pool,
            &entry(
                "/work/other",
                BrainEntryKind::Fact,
                "Other",
                "SQLite is not used here",
            ),
        )
        .await
        .unwrap();
        let mut proposed = entry(
            app,
            BrainEntryKind::Fact,
            "Proposal",
            "SQLite WAL mode is on",
        );
        proposed.status = BrainEntryStatus::Proposed;
        BrainEntry::create(&pool, &proposed).await.unwrap();

        let hits = BrainEntry::search(&pool, app, "\"聊天记录\"", 10)
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.title, "数据库选型");

        let hits = BrainEntry::search(&pool, app, "\"sqlite\"", 10)
            .await
            .unwrap();
        assert_eq!(hits.len(), 1, "other scopes and proposals are not returned");

        // Duplicates are ignored; editing re-indexes.
        assert!(
            BrainEntry::create(
                &pool,
                &entry(
                    app,
                    BrainEntryKind::Rule,
                    "x",
                    "Commit messages use plain engineering language"
                )
            )
            .await
            .unwrap()
            .is_none()
        );
        let rule = &BrainEntry::active_rules(&pool, app, 5).await.unwrap()[0];
        BrainEntry::update(
            &pool,
            rule.id,
            None,
            None,
            Some("Commit subjects stay under 72 characters"),
            None,
            true,
        )
        .await
        .unwrap();
        assert_eq!(
            BrainEntry::search(&pool, app, "\"72 characters\"", 10)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            BrainEntry::find(&pool, rule.id)
                .await
                .unwrap()
                .unwrap()
                .locked
        );
    }

    #[tokio::test]
    async fn deliveries_are_tracked_per_cli_session() {
        let pool = pool().await;
        let e = BrainEntry::create(&pool, &entry("", BrainEntryKind::Fact, "f", "fact body"))
            .await
            .unwrap()
            .unwrap();
        let member = Uuid::new_v4();
        BrainEntry::mark_delivered(&pool, member, Some("cli-1"), &[e.id])
            .await
            .unwrap();
        assert_eq!(
            BrainEntry::delivered_ids(&pool, member, "cli-1")
                .await
                .unwrap(),
            vec![e.id]
        );
        assert!(
            BrainEntry::delivered_ids(&pool, member, "cli-2")
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            BrainEntry::find(&pool, e.id)
                .await
                .unwrap()
                .unwrap()
                .use_count,
            1
        );
    }
}
