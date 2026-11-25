use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// The last message and prompt prefix a member's CLI session has seen.
///
/// Only committed fields are trusted, and only while the member is still on the
/// same CLI session that the cursor was committed for.
#[derive(Debug, Clone, FromRow)]
pub struct ChatMemberContextCursor {
    pub session_agent_id: Uuid,
    pub agent_session_id: Option<String>,
    pub last_message_id: Option<Uuid>,
    pub prefix_hash: Option<String>,
}

impl ChatMemberContextCursor {
    pub async fn find(
        pool: &SqlitePool,
        session_agent_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ChatMemberContextCursor>(
            r#"
            SELECT session_agent_id, agent_session_id, last_message_id, prefix_hash
            FROM chat_member_context_cursors
            WHERE session_agent_id = ?1
            "#,
        )
        .bind(session_agent_id)
        .fetch_optional(pool)
        .await
    }

    /// Record what a starting run is about to show, without trusting it yet.
    pub async fn stage(
        pool: &SqlitePool,
        session_agent_id: Uuid,
        run_id: Uuid,
        last_message_id: Uuid,
        prefix_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO chat_member_context_cursors
                (session_agent_id, pending_run_id, pending_message_id, pending_prefix_hash)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(session_agent_id) DO UPDATE SET
                pending_run_id = excluded.pending_run_id,
                pending_message_id = excluded.pending_message_id,
                pending_prefix_hash = excluded.pending_prefix_hash,
                updated_at = datetime('now', 'subsec')
            "#,
        )
        .bind(session_agent_id)
        .bind(run_id)
        .bind(last_message_id)
        .bind(prefix_hash)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Trust the staged cursor once its run succeeded. A newer run that staged
    /// in the meantime wins, so this only applies to the run that staged last.
    pub async fn commit(
        pool: &SqlitePool,
        session_agent_id: Uuid,
        run_id: Uuid,
        agent_session_id: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE chat_member_context_cursors
            SET agent_session_id = ?3,
                last_message_id = pending_message_id,
                prefix_hash = pending_prefix_hash,
                pending_run_id = NULL,
                pending_message_id = NULL,
                pending_prefix_hash = NULL,
                updated_at = datetime('now', 'subsec')
            WHERE session_agent_id = ?1 AND pending_run_id = ?2
            "#,
        )
        .bind(session_agent_id)
        .bind(run_id)
        .bind(agent_session_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }
}
