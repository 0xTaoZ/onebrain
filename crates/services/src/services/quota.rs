//! Subscription quota of the agent CLIs, read from what they already record
//! locally. No network request and no tokens are spent.
//!
//! Codex writes its rate limits (a short window and a weekly window, each
//! with the percentage used and the reset time) into its session log after
//! every turn; the newest record is the current state.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

/// Only the end of a log is read; the newest record is always there.
const TAIL_BYTES: u64 = 512 * 1024;
const FILES_TO_CHECK: usize = 6;

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct QuotaWindow {
    pub used_percent: f64,
    #[ts(type = "number")]
    pub window_minutes: i64,
    /// Unix seconds when the window resets.
    #[ts(type = "number | null")]
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct RuntimeQuota {
    pub runner: String,
    pub plan: Option<String>,
    pub primary: Option<QuotaWindow>,
    pub secondary: Option<QuotaWindow>,
    pub limit_reached: bool,
    /// When the CLI recorded these numbers (its last turn).
    pub observed_at: Option<String>,
}

fn codex_home() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))
}

/// Newest session logs: walks year/month/day folders from the latest down.
fn newest_logs(sessions: &Path, wanted: usize) -> Vec<PathBuf> {
    fn sorted_dirs(dir: &Path) -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir())
                    .collect()
            })
            .unwrap_or_default();
        dirs.sort();
        dirs.reverse();
        dirs
    }
    let mut found = Vec::new();
    for year in sorted_dirs(sessions) {
        for month in sorted_dirs(&year) {
            for day in sorted_dirs(&month) {
                let mut files: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(&day)
                    .map(|entries| {
                        entries
                            .flatten()
                            .map(|entry| entry.path())
                            .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
                            .filter_map(|path| {
                                let modified = path.metadata().ok()?.modified().ok()?;
                                Some((modified, path))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                files.sort_by(|a, b| b.0.cmp(&a.0));
                found.extend(files.into_iter().map(|(_, path)| path));
                if found.len() >= wanted {
                    found.truncate(wanted);
                    return found;
                }
            }
        }
    }
    found
}

fn tail(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let start = length.saturating_sub(TAIL_BYTES);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn window(value: &Value) -> Option<QuotaWindow> {
    Some(QuotaWindow {
        used_percent: value.get("used_percent")?.as_f64()?,
        window_minutes: value
            .get("window_minutes")
            .and_then(Value::as_i64)
            .unwrap_or(0),
        resets_at: value.get("resets_at").and_then(Value::as_i64),
    })
}

/// Parse the newest `rate_limits` record from Codex log text.
pub fn parse_codex_log(text: &str) -> Option<RuntimeQuota> {
    text.lines().rev().find_map(|line| {
        if !line.contains("\"rate_limits\"") {
            return None;
        }
        let record: Value = serde_json::from_str(line).ok()?;
        let limits = record
            .pointer("/payload/rate_limits")
            .or_else(|| record.get("rate_limits"))?;
        if limits.is_null() {
            return None;
        }
        Some(RuntimeQuota {
            runner: "CODEX".to_string(),
            plan: limits
                .get("plan_type")
                .and_then(Value::as_str)
                .map(str::to_string),
            primary: limits.get("primary").and_then(window),
            secondary: limits.get("secondary").and_then(window),
            limit_reached: limits
                .get("rate_limit_reached_type")
                .is_some_and(|reached| !reached.is_null()),
            observed_at: record
                .get("timestamp")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    })
}

pub fn codex_quota() -> Option<RuntimeQuota> {
    let sessions = codex_home()?.join("sessions");
    newest_logs(&sessions, FILES_TO_CHECK)
        .iter()
        .find_map(|path| parse_codex_log(&tail(path)?))
}

/// Every runtime whose quota can be read on this machine.
pub fn all_quotas() -> Vec<RuntimeQuota> {
    codex_quota().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_newest_rate_limit_record() {
        let log = r#"{"timestamp":"2026-11-27T08:00:00Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":12.0,"window_minutes":300,"resets_at":100},"secondary":null,"plan_type":"plus","rate_limit_reached_type":null}}}
{"timestamp":"2026-11-27T09:00:00Z","type":"event_msg","payload":{"type":"agent_message"}}
{"timestamp":"2026-11-27T09:30:00Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":41.0,"window_minutes":300,"resets_at":200},"secondary":{"used_percent":6.0,"window_minutes":10080,"resets_at":300},"plan_type":"plus","rate_limit_reached_type":null}}}
"#;
        let quota = parse_codex_log(log).unwrap();
        assert_eq!(quota.plan.as_deref(), Some("plus"));
        assert_eq!(quota.primary.unwrap().used_percent, 41.0);
        assert_eq!(quota.secondary.unwrap().window_minutes, 10080);
        assert!(!quota.limit_reached);
        assert_eq!(quota.observed_at.as_deref(), Some("2026-11-27T09:30:00Z"));
    }

    #[test]
    fn finds_logs_newest_day_first() {
        let tmp = tempfile::tempdir().unwrap();
        for day in ["2026/11/26", "2026/11/27"] {
            let dir = tmp.path().join(day);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("a.jsonl"), "{}\n").unwrap();
        }
        let logs = newest_logs(tmp.path(), 1);
        assert!(logs[0].to_string_lossy().contains("2026/11/27"));
        assert!(parse_codex_log("{}\nnot json\n").is_none());
    }
}
