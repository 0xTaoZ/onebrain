use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// The working-tree snapshot taken before one agent run.
#[derive(Debug, Clone, FromRow)]
pub struct ChatRunCheckpoint {
    pub run_id: Uuid,
    pub session_id: Uuid,
    pub workspace_path: String,
    pub checkpoint: String,
    pub after_checkpoint: Option<String>,
}

impl ChatRunCheckpoint {
    pub async fn create(
        pool: &SqlitePool,
        run_id: Uuid,
        session_id: Uuid,
        workspace_path: &str,
        checkpoint: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO chat_run_checkpoints (run_id, session_id, workspace_path, checkpoint) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(run_id)
        .bind(session_id)
        .bind(workspace_path)
        .bind(checkpoint)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_run(pool: &SqlitePool, run_id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ChatRunCheckpoint>(
            "SELECT run_id, session_id, workspace_path, checkpoint, after_checkpoint FROM chat_run_checkpoints WHERE run_id = ?1",
        )
        .bind(run_id)
        .fetch_optional(pool)
        .await
    }

    /// Record the snapshot taken when the run ended. No-op for runs that had
    /// no snapshot at the start (folders outside git).
    pub async fn set_after(
        pool: &SqlitePool,
        run_id: Uuid,
        after_checkpoint: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE chat_run_checkpoints SET after_checkpoint = ?2 WHERE run_id = ?1")
            .bind(run_id)
            .bind(after_checkpoint)
            .execute(pool)
            .await?;
        Ok(())
    }
}
