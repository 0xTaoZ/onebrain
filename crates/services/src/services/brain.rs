//! The team brain: a folder of Markdown files that agents read and write
//! themselves, shared by every vendor's CLI.
//!
//! The folder follows the `AGENTS.md` convention, so Claude Code, Codex,
//! Gemini and others can use it without OneBrain. OneBrain adds what a plain
//! folder lacks: it is a git repository, and after every run the changes are
//! committed with the agent as author. That gives provenance ("which agent
//! wrote this, in which run"), history and restore for free, and the app can
//! show what reading the brain costs each agent in tokens.

use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    process::Command,
};

use serde::Serialize;
use ts_rs::TS;

/// Files the prompt tells every agent to read first.
pub const ALWAYS_READ: &[&str] = &["AGENTS.md", "MEMORY.md"];
const SKIP_DIRS: &[&str] = &[".git", "tmp", "work", "node_modules", ".obsidian"];
const MAX_FILES: usize = 5000;

const AGENTS_MD: &str = "# Team brain

Shared memory for every agent in this team, whatever vendor runs it. Plain
Markdown files; git records who changed what in which run.

## Read
- Read MEMORY.md first: one line per file saying what it holds.
- Then read only the files relevant to your task. Do not read everything.

## Write
- Record only what stays true after this task: decisions and why, rules the
  user gave, project facts that the code does not show, unfinished work.
- One topic per file. Update the existing file instead of adding a duplicate,
  and add or update its line in MEMORY.md.
- decisions/<topic>.md: context, decision, reason. rules/<topic>.md: what the
  user asked for, as short imperative lines. projects/<name>.md: state,
  commands, next step. handoffs/<topic>.md: what is left and where.
- Keep entries short and factual. No command logs, chat transcripts or secrets.
- Do not delete another agent's entry; mark it superseded and say why.
";

const MEMORY_MD: &str = "# Index

One line per file: `path` - what it holds. Keep this list current.

- `AGENTS.md` - how to read and write this brain
";

#[derive(Debug, thiserror::Error)]
pub enum BrainError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git {0} failed: {1}")]
    Git(String, String),
    #[error("path is outside the brain: {0}")]
    UnsafePath(String),
}

/// Where the brain lives: `ONEBRAIN_BRAIN_DIR`, else `~/OneBrain/brain`.
pub fn brain_dir() -> PathBuf {
    if cfg!(test) {
        // Tests never touch the user's brain.
        return std::env::temp_dir().join("onebrain-test-brain");
    }
    if let Some(dir) = std::env::var_os("ONEBRAIN_BRAIN_DIR").filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("OneBrain")
        .join("brain")
}

fn git(dir: &Path, args: &[&str], author: Option<&str>) -> Result<String, BrainError> {
    let mut command = Command::new("git");
    command.arg("-C").arg(dir).args(args);
    let (name, email) = match author {
        Some(name) => (
            name.to_string(),
            format!("{}@agents.onebrain.local", slug(name)),
        ),
        None => ("OneBrain".to_string(), "brain@onebrain.local".to_string()),
    };
    command
        .env("GIT_AUTHOR_NAME", &name)
        .env("GIT_AUTHOR_EMAIL", &email)
        .env("GIT_COMMITTER_NAME", "OneBrain")
        .env("GIT_COMMITTER_EMAIL", "brain@onebrain.local");
    let output = command.output()?;
    if !output.status.success() {
        return Err(BrainError::Git(
            args.first().copied().unwrap_or("").to_string(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn slug(name: &str) -> String {
    let cleaned: String = name
        .trim_start_matches('@')
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    if cleaned.trim_matches('-').is_empty() {
        "agent".into()
    } else {
        cleaned
    }
}

fn is_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// Create the brain on first use. An existing folder is only turned into a
/// git repository when it has none, and its files are never overwritten.
pub fn ensure_brain(dir: &Path) -> Result<(), BrainError> {
    std::fs::create_dir_all(dir)?;
    for (name, text) in [("AGENTS.md", AGENTS_MD), ("MEMORY.md", MEMORY_MD)] {
        let path = dir.join(name);
        if !path.exists() {
            std::fs::write(path, text)?;
        }
    }
    for folder in ["decisions", "rules", "projects", "handoffs"] {
        let path = dir.join(folder);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
            std::fs::write(path.join(".gitkeep"), "")?;
        }
    }
    if !is_repo(dir) {
        git(dir, &["init", "-q", "-b", "main"], None)?;
        commit(dir, None, "Start the team brain")?;
    }
    Ok(())
}

/// Commit everything that changed, as `author`. Returns the new commit, or
/// `None` when nothing changed or the folder is not a git repository.
pub fn commit(
    dir: &Path,
    author: Option<&str>,
    message: &str,
) -> Result<Option<String>, BrainError> {
    if !is_repo(dir) {
        return Ok(None);
    }
    git(dir, &["add", "-A", "--", "."], author)?;
    if git(dir, &["diff", "--cached", "--quiet"], author).is_ok() {
        return Ok(None);
    }
    git(dir, &["commit", "-q", "--no-verify", "-m", message], author)?;
    Ok(Some(
        git(dir, &["rev-parse", "HEAD"], author)?.trim().to_string(),
    ))
}

/// Rough token count: about four Latin characters per token, one per CJK character.
pub fn estimate_tokens(text: &str) -> usize {
    let (ascii, other) = text.chars().fold((0usize, 0usize), |(a, o), c| {
        if c.is_ascii() { (a + 1, o) } else { (a, o + 1) }
    });
    ascii.div_ceil(4) + other
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct BrainFile {
    pub path: String,
    #[ts(type = "number")]
    pub bytes: u64,
    #[ts(type = "number")]
    pub tokens: usize,
    /// Read by every agent at the start of a conversation.
    pub always_read: bool,
    pub last_author: Option<String>,
    pub last_changed: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct BrainChange {
    pub commit: String,
    pub author: String,
    pub at: String,
    pub message: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct BrainOverview {
    pub dir: String,
    pub tracked: bool,
    pub files: Vec<BrainFile>,
    /// Tokens every agent spends reading the always-read files once per conversation.
    #[ts(type = "number")]
    pub always_read_tokens: usize,
    #[ts(type = "number")]
    pub total_tokens: usize,
    pub recent: Vec<BrainChange>,
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct BrainSearchHit {
    pub path: String,
    #[ts(type = "number")]
    pub line: usize,
    pub text: String,
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_FILES {
            return;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                walk(&path, root, out);
            }
        } else if kind.is_file()
            && name.ends_with(".md")
            && let Ok(rel) = path.strip_prefix(root)
        {
            out.push(rel.to_path_buf());
        }
    }
}

fn rel_string(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Parse `git log` output into changes, newest first.
fn log(dir: &Path, limit: usize, path: Option<&str>) -> Vec<BrainChange> {
    if !is_repo(dir) {
        return Vec::new();
    }
    let limit = format!("-n{limit}");
    let mut args = vec![
        "log",
        &limit,
        "--name-only",
        "--format=%x1e%H%x1f%an%x1f%aI%x1f%s",
    ];
    if let Some(path) = path {
        args.extend(["--", path]);
    }
    let Ok(raw) = git(dir, &args, None) else {
        return Vec::new();
    };
    raw.split('\u{1e}')
        .filter_map(|block| {
            let mut lines = block.lines();
            let head = lines.next()?;
            let mut fields = head.split('\u{1f}');
            let commit = fields.next()?.to_string();
            let author = fields.next()?.to_string();
            let at = fields.next()?.to_string();
            let message = fields.next().unwrap_or("").to_string();
            let files = lines
                .filter(|l| !l.trim().is_empty())
                .map(str::to_string)
                .collect();
            Some(BrainChange {
                commit,
                author,
                at,
                message,
                files,
            })
        })
        .collect()
}

pub fn overview(dir: &Path) -> Result<BrainOverview, BrainError> {
    let mut paths = Vec::new();
    walk(dir, dir, &mut paths);
    let recent_long = log(dir, 400, None);
    let mut last: HashMap<&str, (&str, &str)> = HashMap::new();
    for change in &recent_long {
        for file in &change.files {
            last.entry(file.as_str())
                .or_insert((&change.author, &change.at));
        }
    }
    let mut files: Vec<BrainFile> = paths
        .iter()
        .map(|rel| {
            let path = rel_string(rel);
            let text = std::fs::read_to_string(dir.join(rel)).unwrap_or_default();
            let (author, at) = last.get(path.as_str()).copied().unzip();
            BrainFile {
                always_read: ALWAYS_READ.contains(&path.as_str()),
                bytes: text.len() as u64,
                tokens: estimate_tokens(&text),
                last_author: author.map(str::to_string),
                last_changed: at.map(str::to_string),
                path,
            }
        })
        .collect();
    files.sort_by(|a, b| b.always_read.cmp(&a.always_read).then(a.path.cmp(&b.path)));
    Ok(BrainOverview {
        dir: dir.to_string_lossy().to_string(),
        tracked: is_repo(dir),
        always_read_tokens: files
            .iter()
            .filter(|f| f.always_read)
            .map(|f| f.tokens)
            .sum(),
        total_tokens: files.iter().map(|f| f.tokens).sum(),
        recent: recent_long.into_iter().take(30).collect(),
        files,
    })
}

/// Resolve a relative path inside the brain, refusing anything that escapes it.
fn inside(dir: &Path, rel: &str) -> Result<PathBuf, BrainError> {
    let candidate = Path::new(rel);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(BrainError::UnsafePath(rel.to_string()));
    }
    Ok(dir.join(candidate))
}

pub fn read(dir: &Path, rel: &str) -> Result<String, BrainError> {
    Ok(std::fs::read_to_string(inside(dir, rel)?)?)
}

pub fn history(dir: &Path, rel: &str, limit: usize) -> Result<Vec<BrainChange>, BrainError> {
    inside(dir, rel)?;
    Ok(log(dir, limit, Some(rel)))
}

/// The file as it was in `commit`, for showing an old version.
pub fn read_at(dir: &Path, rel: &str, commit: &str) -> Result<String, BrainError> {
    inside(dir, rel)?;
    if commit.is_empty() || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(BrainError::UnsafePath(commit.to_string()));
    }
    git(dir, &["show", &format!("{commit}:{rel}")], None)
}

/// Put one file back to how it was in `commit`, recorded as the user's change.
pub fn restore(dir: &Path, rel: &str, commit: &str) -> Result<Option<String>, BrainError> {
    let text = read_at(dir, rel, commit)?;
    std::fs::write(inside(dir, rel)?, text)?;
    self::commit(
        dir,
        Some("you"),
        &format!("Restore {rel} to {}", &commit[..commit.len().min(8)]),
    )
}

/// Case-insensitive search over every file; Chinese works as plain substrings.
pub fn search(dir: &Path, query: &str, limit: usize) -> Vec<BrainSearchHit> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut paths = Vec::new();
    walk(dir, dir, &mut paths);
    let mut hits = Vec::new();
    for rel in paths {
        let Ok(text) = std::fs::read_to_string(dir.join(&rel)) else {
            continue;
        };
        let path = rel_string(&rel);
        for (index, line) in text.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                hits.push(BrainSearchHit {
                    path: path.clone(),
                    line: index + 1,
                    text: line.chars().take(200).collect(),
                });
                if hits.len() >= limit {
                    return hits;
                }
            }
        }
    }
    hits
}

/// The fixed lines added to an agent's prompt once per conversation.
pub fn prompt_note(dir: &Path) -> String {
    let dir = dir.to_string_lossy();
    format!(
        "## Team Brain\nShared memory for this team at `{dir}`. Read `{dir}/AGENTS.md` and `{dir}/MEMORY.md` before work that depends on earlier decisions, rules or project state. When you settle something that should outlast this task, write it there as AGENTS.md describes.\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_the_brain_and_attributes_changes_to_agents() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("brain");
        ensure_brain(&dir).unwrap();
        assert!(dir.join("AGENTS.md").exists() && dir.join("decisions").is_dir());

        std::fs::write(
            dir.join("decisions/storage.md"),
            "# Storage\nUse SQLite. 不用 Postgres。\n",
        )
        .unwrap();
        assert!(commit(&dir, Some("@claude"), "run 1").unwrap().is_some());
        assert!(
            commit(&dir, Some("@claude"), "nothing").unwrap().is_none(),
            "no change, no commit"
        );

        let view = overview(&dir).unwrap();
        let storage = view
            .files
            .iter()
            .find(|f| f.path == "decisions/storage.md")
            .unwrap();
        assert_eq!(storage.last_author.as_deref(), Some("@claude"));
        assert!(
            view.files[0].always_read,
            "always-read files are listed first"
        );
        assert!(view.always_read_tokens > 0);
        assert_eq!(
            view.recent[0].files,
            vec!["decisions/storage.md".to_string()]
        );

        assert_eq!(search(&dir, "postgres", 10)[0].line, 2);
        assert_eq!(search(&dir, "不用", 10).len(), 1);

        std::fs::write(dir.join("decisions/storage.md"), "broken\n").unwrap();
        commit(&dir, Some("@codex"), "run 2").unwrap();
        let old = history(&dir, "decisions/storage.md", 10).unwrap()[1]
            .commit
            .clone();
        restore(&dir, "decisions/storage.md", &old).unwrap();
        assert!(
            read(&dir, "decisions/storage.md")
                .unwrap()
                .contains("SQLite")
        );
        assert!(read(&dir, "../outside.md").is_err());
    }

    #[test]
    fn an_existing_folder_keeps_its_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("AGENTS.md"), "my own rules\n").unwrap();
        ensure_brain(tmp.path()).unwrap();
        assert_eq!(read(tmp.path(), "AGENTS.md").unwrap(), "my own rules\n");
    }

    #[test]
    fn token_estimate_counts_cjk_per_character() {
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("共享大脑"), 4);
    }
}
