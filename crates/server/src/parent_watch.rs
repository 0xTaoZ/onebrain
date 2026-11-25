//! Shut down a backend whose owning desktop process is gone.
//!
//! A crash or force quit never runs the desktop app's exit handler, so the
//! backend has to notice on its own that it was re-parented. Shutting down
//! through the normal graceful path also stops the agent processes it owns.

use std::time::Duration;

/// Set by the desktop app to its own process id when it spawns the backend.
pub const PARENT_PID_ENV: &str = "OPENTEAMS_PARENT_PID";

const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn parse_parent_pid(value: &str) -> Option<u32> {
    value.trim().parse().ok().filter(|pid| *pid > 1)
}

/// Resolves once the process named by `OPENTEAMS_PARENT_PID` is no longer this
/// process's parent. Never resolves when no owner was named, so standalone and
/// npx runs are unaffected.
pub async fn wait_for_parent_exit() {
    let Some(expected) = std::env::var(PARENT_PID_ENV)
        .ok()
        .and_then(|value| parse_parent_pid(&value))
    else {
        return std::future::pending().await;
    };

    #[cfg(unix)]
    {
        wait_until_parent_changes(expected, std::os::unix::process::parent_id, POLL_INTERVAL)
            .await;
    }

    #[cfg(not(unix))]
    {
        // Windows does not re-parent orphans; ownership there needs a job object.
        let _ = expected;
        std::future::pending::<()>().await;
    }
}

#[cfg_attr(not(unix), allow(dead_code))]
async fn wait_until_parent_changes(
    expected: u32,
    current_parent: impl Fn() -> u32,
    interval: Duration,
) {
    loop {
        let current = current_parent();
        if current != expected {
            tracing::warn!(
                expected_parent_pid = expected,
                current_parent_pid = current,
                "Owning desktop process is gone; shutting down backend"
            );
            return;
        }
        tokio::time::sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    use super::*;

    #[test]
    fn parent_pid_must_name_a_real_owner() {
        assert_eq!(parse_parent_pid(" 4242\n"), Some(4242));
        assert_eq!(parse_parent_pid("1"), None);
        assert_eq!(parse_parent_pid("0"), None);
        assert_eq!(parse_parent_pid("-5"), None);
        assert_eq!(parse_parent_pid("desktop"), None);
    }

    #[tokio::test]
    async fn resolves_when_the_owner_is_replaced() {
        let parent = Arc::new(AtomicU32::new(4242));
        let observed = parent.clone();
        let watch = tokio::spawn(async move {
            wait_until_parent_changes(
                4242,
                move || observed.load(Ordering::SeqCst),
                Duration::from_millis(5),
            )
            .await;
        });

        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(!watch.is_finished(), "watch ended while the owner was alive");

        parent.store(1, Ordering::SeqCst);
        tokio::time::timeout(Duration::from_secs(1), watch)
            .await
            .expect("watch did not notice the re-parenting")
            .unwrap();
    }

    #[tokio::test]
    async fn resolves_immediately_when_the_owner_died_before_startup() {
        tokio::time::timeout(
            Duration::from_millis(100),
            wait_until_parent_changes(4242, || 1, Duration::from_secs(60)),
        )
        .await
        .expect("an already re-parented backend must stop at once");
    }
}
