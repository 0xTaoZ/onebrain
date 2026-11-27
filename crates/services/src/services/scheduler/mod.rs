//! Sends scheduled messages to groups, so agents can work while nobody is at
//! the keyboard.
//!
//! The loop sleeps until the earliest due schedule (or until a schedule is
//! changed), claims each due run with a compare-and-set on `next_run_at` so it
//! fires once even if two loops race, and then posts the prompt through the
//! normal message path: mentions, queues and approvals work exactly as if you
//! had typed it. Nothing is auto-approved. Runs missed while the app was
//! closed are either skipped or run once, never replayed one by one. On macOS
//! the Mac is kept awake (idle sleep only) while any schedule is waiting.

pub mod cron;

use std::{sync::LazyLock, time::Duration as StdDuration};

use chrono::{DateTime, Duration, Local, Utc};
use db::models::{
    chat_message::ChatSenderType,
    chat_schedule::{
        ChatSchedule, ChatScheduleInput, ChatScheduleKind, ChatScheduleMisfire,
        ChatScheduleRunStatus,
    },
    chat_session::ChatSession,
    chat_session_agent::{ChatSessionAgent, ChatSessionAgentState},
};
use sqlx::SqlitePool;
use tokio::sync::Notify;
use uuid::Uuid;

use self::cron::CronSpec;
use super::chat_runner::ChatRunner;

/// A run is "missed" when the app wakes up this long after it was due.
const MISSED_AFTER: i64 = 120;
/// Upper bound on one sleep, so clock changes are noticed.
const MAX_SLEEP_SECONDS: u64 = 300;

static WAKE: LazyLock<Notify> = LazyLock::new(Notify::new);

/// Call after creating, editing or deleting a schedule.
pub fn wake_scheduler() {
    WAKE.notify_one();
}

/// Check a schedule and work out its first run after `now`.
pub fn validate(
    input: &ChatScheduleInput,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, String> {
    if input.title.trim().is_empty() {
        return Err("Give the schedule a name.".into());
    }
    if input.prompt.trim().is_empty() {
        return Err("Write the message to send.".into());
    }
    match input.kind {
        ChatScheduleKind::Cron => {
            let expr = input
                .cron_expr
                .as_deref()
                .ok_or("A cron schedule needs an expression.")?;
            CronSpec::parse(expr)?;
        }
        ChatScheduleKind::Every => {
            if input.every_seconds.is_none_or(|seconds| seconds < 60) {
                return Err("Repeat at most once a minute.".into());
            }
        }
        ChatScheduleKind::Once => {
            input.run_at.ok_or("A one-time schedule needs a time.")?;
        }
    }
    Ok(next_run(
        input.kind,
        input.cron_expr.as_deref(),
        input.every_seconds,
        input.run_at,
        now,
    ))
}

/// Next run strictly after `now`. Cron times are read in this Mac's time zone.
pub fn next_run(
    kind: ChatScheduleKind,
    cron_expr: Option<&str>,
    every_seconds: Option<i64>,
    run_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    match kind {
        ChatScheduleKind::Cron => CronSpec::parse(cron_expr?)
            .ok()?
            .next_after(&now.with_timezone(&Local))
            .map(|time| time.with_timezone(&Utc)),
        ChatScheduleKind::Every => Some(now + Duration::seconds(every_seconds?.max(60))),
        ChatScheduleKind::Once => run_at.filter(|at| *at > now),
    }
}

/// The next few run times, for showing in the editor.
pub fn preview(input: &ChatScheduleInput, now: DateTime<Utc>, count: usize) -> Vec<DateTime<Utc>> {
    let mut times = Vec::new();
    let mut cursor = now;
    while times.len() < count {
        match next_run(
            input.kind,
            input.cron_expr.as_deref(),
            input.every_seconds,
            input.run_at,
            cursor,
        ) {
            Some(next) => {
                times.push(next);
                cursor = next;
            }
            None => break,
        }
    }
    times
}

pub struct Scheduler {
    pool: SqlitePool,
    runner: ChatRunner,
}

impl Scheduler {
    pub fn start(pool: SqlitePool, runner: ChatRunner) {
        let scheduler = Self { pool, runner };
        tokio::spawn(async move { scheduler.run().await });
    }

    async fn run(self) {
        let mut awake = KeepAwake::default();
        loop {
            let now = Utc::now();
            match ChatSchedule::due(&self.pool, now).await {
                Ok(due) => {
                    for schedule in due {
                        self.fire(schedule, now).await;
                    }
                }
                Err(err) => tracing::warn!(error = %err, "failed to read due schedules"),
            }
            awake.set(ChatSchedule::count_enabled(&self.pool).await.unwrap_or(0) > 0);

            let sleep = match ChatSchedule::earliest_next(&self.pool).await {
                Ok(Some(next)) => (next - Utc::now())
                    .to_std()
                    .unwrap_or(StdDuration::ZERO)
                    .min(StdDuration::from_secs(MAX_SLEEP_SECONDS)),
                _ => StdDuration::from_secs(MAX_SLEEP_SECONDS),
            };
            tokio::select! {
                _ = tokio::time::sleep(sleep) => {}
                _ = WAKE.notified() => {}
            }
        }
    }

    async fn fire(&self, schedule: ChatSchedule, now: DateTime<Utc>) {
        let Some(due_at) = schedule.next_run_at else {
            return;
        };
        let new_next = next_run(
            schedule.kind,
            schedule.cron_expr.as_deref(),
            schedule.every_seconds,
            schedule.run_at,
            now,
        );
        match ChatSchedule::claim(&self.pool, schedule.id, due_at, new_next, now).await {
            Ok(true) => {}
            Ok(false) => return, // someone else took it, or it was edited
            Err(err) => {
                tracing::warn!(schedule_id = %schedule.id, error = %err, "failed to claim schedule");
                return;
            }
        }
        let status = self.send(&schedule, due_at, now).await;
        let (status, message_id) = match status {
            Ok(outcome) => outcome,
            Err(err) => {
                tracing::warn!(schedule_id = %schedule.id, error = %err, "scheduled message failed");
                (ChatScheduleRunStatus::Failed, None)
            }
        };
        let _ = ChatSchedule::record_result(&self.pool, schedule.id, status, message_id).await;
    }

    async fn send(
        &self,
        schedule: &ChatSchedule,
        due_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(ChatScheduleRunStatus, Option<Uuid>), String> {
        if schedule.misfire == ChatScheduleMisfire::Skip
            && (now - due_at).num_seconds() > MISSED_AFTER
        {
            return Ok((ChatScheduleRunStatus::SkippedMissed, None));
        }
        let busy = ChatSessionAgent::find_all_for_session(&self.pool, schedule.session_id)
            .await
            .map_err(|err| err.to_string())?
            .iter()
            .any(|member| {
                matches!(
                    member.state,
                    ChatSessionAgentState::Running | ChatSessionAgentState::Stopping
                )
            });
        if busy {
            // Do not pile up work on a group that is still busy with the last run.
            return Ok((ChatScheduleRunStatus::SkippedBusy, None));
        }
        let session = ChatSession::find_by_id(&self.pool, schedule.session_id)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("the group no longer exists")?;
        let meta = serde_json::json!({
            // Same key for the same due time, so a retry never sends twice.
            "opKey": format!("schedule:{}:{}", schedule.id, due_at.timestamp()),
            "scheduled": { "schedule_id": schedule.id, "title": schedule.title },
        });
        let result = self
            .runner
            .persist_and_dispatch_message(
                &session,
                ChatSenderType::User,
                None,
                schedule.prompt.clone(),
                Some(meta),
                Uuid::new_v4(),
            )
            .await
            .map_err(|err| err.to_string())?;
        Ok((ChatScheduleRunStatus::Sent, Some(result.message.id)))
    }
}

/// Keeps the Mac from idle-sleeping while schedules are waiting, by holding a
/// `caffeinate` child tied to this process. Dropped when none are left.
#[derive(Default)]
struct KeepAwake {
    child: Option<tokio::process::Child>,
}

impl KeepAwake {
    fn set(&mut self, wanted: bool) {
        if !cfg!(target_os = "macos") {
            return;
        }
        if let Some(child) = &mut self.child
            && child.try_wait().ok().flatten().is_some()
        {
            self.child = None;
        }
        match (wanted, self.child.is_some()) {
            (true, false) => {
                self.child = tokio::process::Command::new("/usr/bin/caffeinate")
                    .args(["-i", "-w", &std::process::id().to_string()])
                    .kill_on_drop(true)
                    .spawn()
                    .ok();
            }
            (false, true) => {
                if let Some(mut child) = self.child.take() {
                    let _ = child.start_kill();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(kind: ChatScheduleKind) -> ChatScheduleInput {
        ChatScheduleInput {
            title: "Nightly review".into(),
            prompt: "@codex review today's changes".into(),
            kind,
            cron_expr: None,
            every_seconds: None,
            run_at: None,
            misfire: None,
            enabled: None,
        }
    }

    #[test]
    fn validation_explains_what_is_missing() {
        let now = Utc::now();
        let mut every = input(ChatScheduleKind::Every);
        every.every_seconds = Some(30);
        assert!(validate(&every, now).unwrap_err().contains("minute"));
        every.every_seconds = Some(3600);
        assert_eq!(
            validate(&every, now).unwrap(),
            Some(now + Duration::hours(1))
        );

        let mut cron = input(ChatScheduleKind::Cron);
        cron.cron_expr = Some("61 * * * *".into());
        assert!(validate(&cron, now).is_err());

        let mut once = input(ChatScheduleKind::Once);
        once.run_at = Some(now - Duration::minutes(1));
        assert_eq!(
            validate(&once, now).unwrap(),
            None,
            "a time in the past never runs"
        );
    }

    #[test]
    fn preview_lists_upcoming_runs_in_order() {
        let mut every = input(ChatScheduleKind::Every);
        every.every_seconds = Some(600);
        let now = Utc::now();
        let times = preview(&every, now, 3);
        assert_eq!(
            times,
            vec![
                now + Duration::minutes(10),
                now + Duration::minutes(20),
                now + Duration::minutes(30)
            ]
        );
        let mut cron = input(ChatScheduleKind::Cron);
        cron.cron_expr = Some("0 9 * * *".into());
        let days = preview(&cron, now, 3);
        assert_eq!(days.len(), 3);
        assert!(days.windows(2).all(|w| w[1] - w[0] >= Duration::hours(23)));
    }
}
