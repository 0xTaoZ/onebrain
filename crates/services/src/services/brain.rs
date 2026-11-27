//! Recall from the team's shared brain for one agent turn.
//!
//! No embedding model and no extra model call: the task text is turned into a
//! trigram full-text query, SQLite ranks matching entries with bm25, and the
//! result is re-ranked by how often an entry proved useful and how recently.
//! Rules are always included. Entries an agent's current CLI session was
//! already given are skipped, so a long conversation pays for each memory
//! once. The rendered block has hard caps on entries and bytes.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use db::models::brain_entry::{BrainEntry, BrainEntryKind};
use sqlx::SqlitePool;
use uuid::Uuid;

const MAX_ENTRIES: usize = 6;
const MAX_RULES: i64 = 3;
const MAX_BYTES: usize = 1800;
const MAX_TERMS: usize = 24;
const CANDIDATES: i64 = 24;
/// After this many days without use an entry counts half as much.
const HALF_LIFE_DAYS: f64 = 45.0;

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF)
}

/// Build an FTS5 trigram query (`"abc" OR "def"`) from free text. Latin words
/// of three or more characters are used as they are; runs of CJK characters
/// are cut into overlapping three-character windows.
pub fn fts_query(text: &str) -> Option<String> {
    const STOP: &[&str] = &[
        "the", "and", "for", "with", "this", "that", "from", "have", "you", "are", "please", "can",
        "der", "die", "das", "und", "mit", "ist", "bitte", "eine",
    ];
    // Split into runs: Latin/digit words and CJK runs, everything else separates.
    let mut runs: Vec<(bool, String)> = Vec::new();
    for c in text.chars() {
        let class = if is_cjk(c) {
            Some(true)
        } else if c.is_alphanumeric() || c == '_' {
            Some(false)
        } else {
            None
        };
        match (class, runs.last_mut()) {
            (Some(cjk), Some((last_cjk, run))) if *last_cjk == cjk && !run.is_empty() => {
                run.push(c)
            }
            (Some(cjk), _) => runs.push((cjk, c.to_string())),
            (None, _) => runs.push((false, String::new())),
        }
    }
    let mut seen = HashSet::new();
    let mut terms = Vec::new();
    for (cjk, run) in runs {
        let candidates: Vec<String> = if cjk {
            let chars: Vec<char> = run.chars().collect();
            chars
                .windows(3)
                .map(|window| window.iter().collect())
                .collect()
        } else {
            let lower = run.to_lowercase();
            if lower.chars().count() >= 3 && !STOP.contains(&lower.as_str()) {
                vec![lower]
            } else {
                Vec::new()
            }
        };
        for term in candidates {
            if terms.len() < MAX_TERMS && seen.insert(term.clone()) {
                terms.push(term);
            }
        }
    }
    if terms.is_empty() {
        return None;
    }
    Some(
        terms
            .iter()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR "),
    )
}

fn weight(entry: &BrainEntry, now: DateTime<Utc>) -> f64 {
    let last = entry.last_used_at.as_deref().unwrap_or(&entry.created_at);
    let age_days = DateTime::parse_from_rfc3339(last)
        .map(|time| (now - time.with_timezone(&Utc)).num_seconds().max(0) as f64 / 86_400.0)
        .unwrap_or(0.0);
    let recency = 0.5_f64.powf(age_days / HALF_LIFE_DAYS);
    let usefulness = 1.0 + (1.0 + entry.use_count as f64).ln() * 0.25;
    let pinned = if entry.locked { 1.3 } else { 1.0 };
    (0.4 + 0.6 * recency) * usefulness * pinned
}

pub struct Recall {
    pub text: String,
    pub ids: Vec<Uuid>,
}

/// Pick the entries worth sending with this task and render them.
pub async fn recall(
    pool: &SqlitePool,
    scope: &str,
    task: &str,
    session_agent_id: Uuid,
    agent_session_id: Option<&str>,
) -> Result<Option<Recall>, sqlx::Error> {
    let delivered: HashSet<Uuid> = match agent_session_id {
        Some(id) => BrainEntry::delivered_ids(pool, session_agent_id, id)
            .await?
            .into_iter()
            .collect(),
        None => HashSet::new(),
    };
    let now = Utc::now();
    let mut picked: Vec<BrainEntry> = BrainEntry::active_rules(pool, scope, MAX_RULES)
        .await?
        .into_iter()
        .filter(|entry| !delivered.contains(&entry.id))
        .collect();
    if let Some(query) = fts_query(task) {
        let mut hits = BrainEntry::search(pool, scope, &query, CANDIDATES).await?;
        // bm25 is negative; more negative is a better match.
        hits.sort_by(|a, b| {
            let score =
                |hit: &db::models::brain_entry::BrainHit| -hit.rank * weight(&hit.entry, now);
            score(b).total_cmp(&score(a))
        });
        for hit in hits {
            if picked.len() >= MAX_ENTRIES {
                break;
            }
            if !delivered.contains(&hit.entry.id) && picked.iter().all(|e| e.id != hit.entry.id) {
                picked.push(hit.entry);
            }
        }
    }
    if picked.is_empty() {
        return Ok(None);
    }
    let mut text = String::new();
    let mut ids = Vec::new();
    for entry in &picked {
        let label = match entry.kind {
            BrainEntryKind::Decision => "decision",
            BrainEntryKind::Rule => "rule",
            BrainEntryKind::Preference => "preference",
            BrainEntryKind::Fact => "fact",
        };
        let line = format!(
            "- [{label}] {}: {}\n",
            entry.title.trim(),
            entry.body.trim().replace('\n', " ")
        );
        if text.len() + line.len() > MAX_BYTES {
            break;
        }
        text.push_str(&line);
        ids.push(entry.id);
    }
    if ids.is_empty() {
        return Ok(None);
    }
    BrainEntry::mark_delivered(pool, session_agent_id, agent_session_id, &ids).await?;
    Ok(Some(Recall { text, ids }))
}

/// Turn a `record` an agent wrote into a proposal: the first line is the
/// title, and a leading `rule:`/`decision:`/`preference:` tag sets the kind.
pub fn proposal_from_record(content: &str) -> Option<(BrainEntryKind, String, String)> {
    let trimmed = content.trim();
    if trimmed.chars().count() < 8 {
        return None;
    }
    let (kind, rest) = [
        ("rule:", BrainEntryKind::Rule),
        ("decision:", BrainEntryKind::Decision),
        ("preference:", BrainEntryKind::Preference),
        ("fact:", BrainEntryKind::Fact),
        ("规则：", BrainEntryKind::Rule),
        ("决定：", BrainEntryKind::Decision),
        ("偏好：", BrainEntryKind::Preference),
    ]
    .iter()
    .find_map(|(tag, kind)| {
        trimmed
            .get(..tag.len())
            .filter(|head| head.eq_ignore_ascii_case(tag))
            .map(|_| (*kind, trimmed[tag.len()..].trim_start()))
    })
    .unwrap_or((BrainEntryKind::Fact, trimmed));
    let first = rest.lines().next().unwrap_or(rest).trim();
    let title: String = first.chars().take(80).collect();
    Some((kind, title, rest.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_mix_latin_words_and_cjk_trigrams() {
        let query = fts_query("请帮我把数据库换成 SQLite, and fix the WAL setting").unwrap();
        assert!(query.contains("\"sqlite\""));
        assert!(query.contains("\"wal\""));
        assert!(query.contains("\"数据库\""));
        assert!(!query.contains("\"the\""), "stop words are dropped");
        assert!(fts_query("ok").is_none());
    }

    #[test]
    fn records_become_typed_proposals() {
        let (kind, title, body) =
            proposal_from_record("Rule: never push to main\nUse a branch.").unwrap();
        assert_eq!(kind, BrainEntryKind::Rule);
        assert_eq!(title, "never push to main");
        assert!(body.ends_with("Use a branch."));
        let (kind, _, _) = proposal_from_record("决定：前端只用暖白主题").unwrap();
        assert_eq!(kind, BrainEntryKind::Decision);
        assert!(proposal_from_record("ok").is_none());
    }
}
