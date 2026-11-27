use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type, types::Json};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, TS)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(use_ts_enum)]
pub enum ChatAuditKind {
    ApprovalRequested,
    ApprovalResolved,
    RunUndone,
}

/// One entry of a group's activity log.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct ChatAuditEvent {
    pub id: Uuid,
    pub session_id: Uuid,
    pub session_agent_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub kind: ChatAuditKind,
    pub dangerous: bool,
    pub summary: String,
    #[ts(type = "JsonValue")]
    pub detail: Json<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct NewChatAuditEvent {
    pub session_id: Uuid,
    pub session_agent_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub kind: ChatAuditKind,
    pub dangerous: bool,
    pub summary: String,
    pub detail: serde_json::Value,
}

const RETENTION_DAYS: i64 = 90;

impl ChatAuditEvent {
    pub async fn record(pool: &SqlitePool, event: NewChatAuditEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO chat_audit_events (id, session_id, session_agent_id, run_id, kind, dangerous, summary, detail) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(Uuid::new_v4())
        .bind(event.session_id)
        .bind(event.session_agent_id)
        .bind(event.run_id)
        .bind(event.kind)
        .bind(event.dangerous)
        .bind(event.summary)
        .bind(Json(event.detail))
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Newest first. Pass the `created_at` and `id` of the last entry seen to
    /// get the next page (keyset paging, so a long log stays fast).
    pub async fn list(
        pool: &SqlitePool,
        session_id: Uuid,
        before: Option<(&str, Uuid)>,
        dangerous_only: bool,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let (before_at, before_id) = match before {
            Some((at, id)) => (Some(at.to_string()), Some(id)),
            None => (None, None),
        };
        sqlx::query_as::<_, Self>(
            "SELECT id, session_id, session_agent_id, run_id, kind, dangerous, summary, detail, created_at \
             FROM chat_audit_events \
             WHERE session_id = ?1 \
               AND (?2 IS NULL OR created_at < ?2 OR (created_at = ?2 AND id < ?3)) \
               AND (?4 = 0 OR dangerous = 1) \
             ORDER BY created_at DESC, id DESC LIMIT ?5",
        )
        .bind(session_id)
        .bind(before_at)
        .bind(before_id)
        .bind(dangerous_only)
        .bind(limit.clamp(1, 200))
        .fetch_all(pool)
        .await
    }

    pub async fn prune(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM chat_audit_events WHERE created_at < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?1)",
        )
        .bind(format!("-{RETENTION_DAYS} days"))
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
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
        sqlx::query("CREATE TABLE chat_sessions (id BLOB PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../../migrations/20261127150000_add_chat_audit_events.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    fn event(session_id: Uuid, summary: &str, dangerous: bool) -> NewChatAuditEvent {
        NewChatAuditEvent {
            session_id,
            session_agent_id: None,
            run_id: None,
            kind: ChatAuditKind::ApprovalRequested,
            dangerous,
            summary: summary.to_string(),
            detail: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn pages_newest_first_filters_dangerous_and_prunes_old_entries() {
        let pool = pool().await;
        let session = Uuid::new_v4();
        let other = Uuid::new_v4();
        for id in [session, other] {
            sqlx::query("INSERT INTO chat_sessions (id) VALUES (?1)")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();
        }
        for i in 0..5 {
            ChatAuditEvent::record(&pool, event(session, &format!("e{i}"), i % 2 == 0))
                .await
                .unwrap();
        }
        ChatAuditEvent::record(&pool, event(other, "elsewhere", true))
            .await
            .unwrap();

        let first = ChatAuditEvent::list(&pool, session, None, false, 3)
            .await
            .unwrap();
        assert_eq!(first.len(), 3);
        let last = first.last().unwrap();
        let rest =
            ChatAuditEvent::list(&pool, session, Some((&last.created_at, last.id)), false, 3)
                .await
                .unwrap();
        assert_eq!(rest.len(), 2);
        let mut seen: Vec<String> = first
            .iter()
            .chain(&rest)
            .map(|e| e.summary.clone())
            .collect();
        seen.sort();
        assert_eq!(seen, vec!["e0", "e1", "e2", "e3", "e4"]);

        let dangerous = ChatAuditEvent::list(&pool, session, None, true, 50)
            .await
            .unwrap();
        assert_eq!(dangerous.len(), 3);
        assert!(dangerous.iter().all(|e| e.dangerous));

        sqlx::query("UPDATE chat_audit_events SET created_at = '2000-01-01T00:00:00.000Z' WHERE summary = 'e0'")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(ChatAuditEvent::prune(&pool).await.unwrap(), 1);
    }
}
