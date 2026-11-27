use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, TS)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(use_ts_enum)]
pub enum ChatScheduleKind {
    Cron,
    Every,
    Once,
}

/// What to do when the app was closed at the scheduled time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, TS)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(use_ts_enum)]
pub enum ChatScheduleMisfire {
    Skip,
    RunOnce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, TS)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(use_ts_enum)]
pub enum ChatScheduleRunStatus {
    Sent,
    SkippedBusy,
    SkippedMissed,
    Failed,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct ChatSchedule {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub prompt: String,
    pub kind: ChatScheduleKind,
    pub cron_expr: Option<String>,
    #[ts(type = "number | null")]
    pub every_seconds: Option<i64>,
    #[ts(type = "string | null")]
    pub run_at: Option<DateTime<Utc>>,
    pub misfire: ChatScheduleMisfire,
    pub enabled: bool,
    #[ts(type = "string | null")]
    pub next_run_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_status: Option<ChatScheduleRunStatus>,
    pub last_message_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
pub struct ChatScheduleInput {
    pub title: String,
    pub prompt: String,
    pub kind: ChatScheduleKind,
    pub cron_expr: Option<String>,
    #[ts(type = "number | null")]
    pub every_seconds: Option<i64>,
    #[ts(type = "string | null")]
    pub run_at: Option<DateTime<Utc>>,
    pub misfire: Option<ChatScheduleMisfire>,
    pub enabled: Option<bool>,
}

const COLUMNS: &str = "id, session_id, title, prompt, kind, cron_expr, every_seconds, run_at, misfire, \
     enabled, next_run_at, last_run_at, last_status, last_message_id, created_at";

impl ChatSchedule {
    pub async fn create(
        pool: &SqlitePool,
        session_id: Uuid,
        input: &ChatScheduleInput,
        next_run_at: Option<DateTime<Utc>>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "INSERT INTO chat_schedules (id, session_id, title, prompt, kind, cron_expr, every_seconds, run_at, misfire, enabled, next_run_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) RETURNING {COLUMNS}"
        ))
        .bind(Uuid::new_v4())
        .bind(session_id)
        .bind(&input.title)
        .bind(&input.prompt)
        .bind(input.kind)
        .bind(&input.cron_expr)
        .bind(input.every_seconds)
        .bind(input.run_at)
        .bind(input.misfire.unwrap_or(ChatScheduleMisfire::RunOnce))
        .bind(input.enabled.unwrap_or(true))
        .bind(next_run_at)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        input: &ChatScheduleInput,
        next_run_at: Option<DateTime<Utc>>,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "UPDATE chat_schedules SET title = ?2, prompt = ?3, kind = ?4, cron_expr = ?5, every_seconds = ?6, \
             run_at = ?7, misfire = ?8, enabled = ?9, next_run_at = ?10, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1 RETURNING {COLUMNS}"
        ))
        .bind(id)
        .bind(&input.title)
        .bind(&input.prompt)
        .bind(input.kind)
        .bind(&input.cron_expr)
        .bind(input.every_seconds)
        .bind(input.run_at)
        .bind(input.misfire.unwrap_or(ChatScheduleMisfire::RunOnce))
        .bind(input.enabled.unwrap_or(true))
        .bind(next_run_at)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT {COLUMNS} FROM chat_schedules WHERE id = ?1"
        ))
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_for_session(
        pool: &SqlitePool,
        session_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT {COLUMNS} FROM chat_schedules WHERE session_id = ?1 ORDER BY created_at ASC"
        ))
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<bool, sqlx::Error> {
        Ok(sqlx::query("DELETE FROM chat_schedules WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected()
            > 0)
    }

    /// Enabled schedules whose time has come.
    pub async fn due(pool: &SqlitePool, now: DateTime<Utc>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT {COLUMNS} FROM chat_schedules WHERE enabled = 1 AND next_run_at IS NOT NULL \
             AND next_run_at <= ?1 ORDER BY next_run_at ASC LIMIT 100"
        ))
        .bind(now)
        .fetch_all(pool)
        .await
    }

    /// When the scheduler has to wake up next.
    pub async fn earliest_next(pool: &SqlitePool) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            "SELECT MIN(next_run_at) FROM chat_schedules WHERE enabled = 1 AND next_run_at IS NOT NULL",
        )
        .fetch_one(pool)
        .await
    }

    pub async fn count_enabled(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_schedules WHERE enabled = 1 AND next_run_at IS NOT NULL",
        )
        .fetch_one(pool)
        .await
    }

    /// Take one due run: moves `next_run_at` forward only if nobody else did
    /// first (compare-and-set), so a run can never fire twice.
    pub async fn claim(
        pool: &SqlitePool,
        id: Uuid,
        expected_next: DateTime<Utc>,
        new_next: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE chat_schedules SET next_run_at = ?3, last_run_at = ?4, \
             enabled = CASE WHEN ?3 IS NULL THEN 0 ELSE enabled END \
             WHERE id = ?1 AND enabled = 1 AND next_run_at = ?2",
        )
        .bind(id)
        .bind(expected_next)
        .bind(new_next)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    /// Make a schedule due right away ("run now").
    pub async fn run_now(
        pool: &SqlitePool,
        id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        Ok(
            sqlx::query("UPDATE chat_schedules SET next_run_at = ?2, enabled = 1 WHERE id = ?1")
                .bind(id)
                .bind(now)
                .execute(pool)
                .await?
                .rows_affected()
                > 0,
        )
    }

    pub async fn record_result(
        pool: &SqlitePool,
        id: Uuid,
        status: ChatScheduleRunStatus,
        message_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat_schedules SET last_status = ?2, last_message_id = COALESCE(?3, last_message_id) WHERE id = ?1",
        )
        .bind(id)
        .bind(status)
        .bind(message_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    async fn pool_with_session() -> (SqlitePool, Uuid) {
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
            "../../migrations/20261127160000_add_chat_schedules.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let session = Uuid::new_v4();
        sqlx::query("INSERT INTO chat_sessions (id) VALUES (?1)")
            .bind(session)
            .execute(&pool)
            .await
            .unwrap();
        (pool, session)
    }

    #[tokio::test]
    async fn a_due_run_can_be_claimed_only_once() {
        let (pool, session) = pool_with_session().await;
        let now = Utc::now();
        let due_at = now - Duration::seconds(5);
        let input = ChatScheduleInput {
            title: "Hourly".into(),
            prompt: "@claude check CI".into(),
            kind: ChatScheduleKind::Every,
            cron_expr: None,
            every_seconds: Some(3600),
            run_at: None,
            misfire: None,
            enabled: None,
        };
        let schedule = ChatSchedule::create(&pool, session, &input, Some(due_at))
            .await
            .unwrap();
        assert_eq!(ChatSchedule::due(&pool, now).await.unwrap().len(), 1);

        let next = Some(now + Duration::hours(1));
        assert!(
            ChatSchedule::claim(&pool, schedule.id, due_at, next, now)
                .await
                .unwrap()
        );
        assert!(
            !ChatSchedule::claim(&pool, schedule.id, due_at, next, now)
                .await
                .unwrap()
        );
        assert!(ChatSchedule::due(&pool, now).await.unwrap().is_empty());
        assert_eq!(ChatSchedule::earliest_next(&pool).await.unwrap(), next);

        // A one-time schedule turns itself off once claimed.
        let once = ChatSchedule::create(
            &pool,
            session,
            &ChatScheduleInput {
                kind: ChatScheduleKind::Once,
                every_seconds: None,
                run_at: Some(due_at),
                ..input.clone()
            },
            Some(due_at),
        )
        .await
        .unwrap();
        assert!(
            ChatSchedule::claim(&pool, once.id, due_at, None, now)
                .await
                .unwrap()
        );
        assert!(
            !ChatSchedule::find_by_id(&pool, once.id)
                .await
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(ChatSchedule::count_enabled(&pool).await.unwrap(), 1);
    }
}
