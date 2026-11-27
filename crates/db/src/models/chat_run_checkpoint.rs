use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// The working-tree snapshot taken before one agent run.
#[derive(Debug, Clone, FromRow)]
pub struct ChatRunCheckpoint {
    pub run_id: Uuid,
    pub session_id: Uuid,
    pub workspace_path: String,
    pub checkpoint: String,
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
            "SELECT run_id, session_id, workspace_path, checkpoint FROM chat_run_checkpoints WHERE run_id = ?1",
        )
        .bind(run_id)
        .fetch_optional(pool)
        .await
    }
}
