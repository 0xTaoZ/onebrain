use std::{sync::Mutex, time::Duration};

use tauri::api::process::CommandChild;

/// Environment variable the backend watches to stop when this app is gone.
pub(crate) const PARENT_PID_ENV: &str = "OPENTEAMS_PARENT_PID";

/// Time the backend gets to stop its agent processes before it is killed.
const STOP_GRACE: Duration = Duration::from_secs(5);

/// Own the backend until final exit; a canceled exit request must keep it alive.
pub(crate) struct BackendState {
    child: Mutex<Option<CommandChild>>,
}

impl BackendState {
    pub(crate) fn new(child: CommandChild) -> Self {
        Self {
            child: Mutex::new(Some(child)),
        }
    }

    /// Ask the backend to shut down, so it can stop the agent processes it
    /// started, and kill it only if it does not exit within the grace period.
    pub(crate) fn stop(&self) {
        self.stop_within(STOP_GRACE);
    }

    fn stop_within(&self, grace: Duration) {
        // Even a poisoned lock must not leave an owned process behind on exit.
        let child = self
            .child
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        let Some(child) = child else {
            return;
        };

        #[cfg(unix)]
        if graceful::terminate(child.pid(), grace) {
            return;
        }
        #[cfg(not(unix))]
        let _ = grace;

        if let Err(error) = child.kill() {
            eprintln!("Failed to stop desktop backend: {error}");
        }
    }
}

impl Drop for BackendState {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(unix)]
mod graceful {
    use std::{
        thread,
        time::{Duration, Instant},
    };

    const POLL_INTERVAL: Duration = Duration::from_millis(25);

    /// Send SIGTERM and wait for the process to disappear. Returns false when
    /// the signal could not be sent or the process outlived the grace period.
    pub(super) fn terminate(pid: u32, grace: Duration) -> bool {
        let Ok(pid) = libc::pid_t::try_from(pid) else {
            return false;
        };
        // SAFETY: kill only sends a signal to a pid; it touches no memory.
        if unsafe { libc::kill(pid, libc::SIGTERM) } != 0 {
            return false;
        }

        // Tauri's process reader thread reaps the child, so an exited backend
        // stops answering signal 0 instead of lingering as a zombie.
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            // SAFETY: signal 0 only checks that the pid exists.
            if unsafe { libc::kill(pid, 0) } != 0 {
                return true;
            }
            thread::sleep(POLL_INTERVAL);
        }
        false
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{path::PathBuf, process::Command, thread, time::Duration};

    use tauri::api::process::Command as SidecarCommand;

    use super::BackendState;

    fn backend(script: &str) -> (BackendState, u32) {
        let (_events, child) = SidecarCommand::new("/bin/sh")
            .args(["-c", script])
            .spawn()
            .unwrap();
        let pid = child.pid();
        (BackendState::new(child), pid)
    }

    fn sleeping_backend() -> (BackendState, u32) {
        backend("exec /bin/sleep 30")
    }

    fn marker_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("openteams-{name}-{}", std::process::id()))
    }

    fn assert_stopped(pid: u32) {
        for _ in 0..100 {
            if !Command::new("/bin/kill")
                .args(["-0", &pid.to_string()])
                .output()
                .unwrap()
                .status
                .success()
            {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("owned child {pid} survived cleanup");
    }

    #[test]
    fn final_cleanup_stops_the_owned_child_and_is_idempotent() {
        let (state, pid) = sleeping_backend();
        state.stop();
        state.stop();
        assert_stopped(pid);
    }

    #[test]
    fn dropping_state_stops_the_owned_child() {
        let (state, pid) = sleeping_backend();
        drop(state);
        assert_stopped(pid);
    }

    #[test]
    fn stop_lets_the_backend_run_its_own_shutdown_first() {
        let marker = marker_path("graceful");
        let _ = std::fs::remove_file(&marker);
        let script = format!(
            "trap 'echo cleaned > \"{}\"; exit 0' TERM; while :; do /bin/sleep 0.05; done",
            marker.display()
        );
        let (state, pid) = backend(&script);
        thread::sleep(Duration::from_millis(100));

        state.stop_within(Duration::from_secs(5));

        assert_stopped(pid);
        let written = std::fs::read_to_string(&marker).unwrap_or_default();
        let _ = std::fs::remove_file(&marker);
        assert_eq!(
            written.trim(),
            "cleaned",
            "backend was killed before SIGTERM cleanup"
        );
    }

    #[test]
    fn stop_kills_a_backend_that_ignores_shutdown() {
        let (state, pid) = backend("trap '' TERM; while :; do /bin/sleep 0.05; done");
        thread::sleep(Duration::from_millis(100));

        state.stop_within(Duration::from_millis(200));

        assert_stopped(pid);
    }
}
