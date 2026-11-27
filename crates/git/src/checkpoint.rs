//! Snapshots of a working tree taken before an agent run, and restoring them.
//!
//! A checkpoint captures every file git would see (tracked files plus new
//! untracked files, honouring `.gitignore`) as a tree object, wraps it in a
//! commit and keeps it under `refs/onebrain/checkpoints/<label>`. Storage is
//! content-addressed, so only changed files cost space, and taking or restoring
//! a checkpoint never touches the user's branch, index or history: both use a
//! private temporary index.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("git {args} failed: {stderr}")]
    Git { args: String, stderr: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("checkpoint label is not valid: {0}")]
    InvalidLabel(String),
    #[error("refusing to remove a path outside the working tree: {0}")]
    UnsafePath(String),
    #[error("files changed after the run: {}", .0.join(", "))]
    Conflict(Vec<String>),
}

fn git(repo: &Path, index: Option<&Path>, args: &[&str]) -> Result<String, CheckpointError> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo).args(args);
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    // Commits made here are internal snapshots; do not depend on user identity.
    command
        .env("GIT_AUTHOR_NAME", "OneBrain")
        .env("GIT_AUTHOR_EMAIL", "checkpoint@onebrain.local")
        .env("GIT_COMMITTER_NAME", "OneBrain")
        .env("GIT_COMMITTER_EMAIL", "checkpoint@onebrain.local");
    let output = command.output()?;
    if !output.status.success() {
        return Err(CheckpointError::Git {
            args: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

fn git_stdin(
    repo: &Path,
    index: Option<&Path>,
    args: &[&str],
    input: &[u8],
) -> Result<String, CheckpointError> {
    use std::{io::Write, process::Stdio};
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    let mut child = command.spawn()?;
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(CheckpointError::Git {
            args: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

fn is_work_tree(repo: &Path) -> bool {
    git(repo, None, &["rev-parse", "--is-inside-work-tree"]).is_ok_and(|out| out == "true")
}

/// A private index file inside the repository's git directory.
fn scratch_index(repo: &Path, name: &str) -> Result<PathBuf, CheckpointError> {
    let path = git(repo, None, &["rev-parse", "--git-path", name])?;
    let path = PathBuf::from(path);
    Ok(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}

fn nul_list(raw: &str) -> HashSet<String> {
    raw.split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

/// Snapshot the working tree of `repo`. Returns `Ok(None)` when `repo` is not
/// inside a git working tree, so callers can skip non-git folders silently.
pub fn create_checkpoint(repo: &Path, label: &str) -> Result<Option<String>, CheckpointError> {
    if label.is_empty() || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(CheckpointError::InvalidLabel(label.to_string()));
    }
    if !is_work_tree(repo) {
        return Ok(None);
    }
    let index = scratch_index(repo, &format!("onebrain-checkpoint-{label}.index"))?;
    let result = (|| {
        git(repo, Some(&index), &["add", "-A", "--", "."])?;
        let tree = git(repo, Some(&index), &["write-tree"])?;
        let message = format!("OneBrain checkpoint {label}");
        let commit = match git(repo, None, &["rev-parse", "--verify", "--quiet", "HEAD"]) {
            Ok(head) if !head.is_empty() => git(
                repo,
                None,
                &["commit-tree", &tree, "-p", &head, "-m", &message],
            )?,
            _ => git(repo, None, &["commit-tree", &tree, "-m", &message])?,
        };
        git(
            repo,
            None,
            &[
                "update-ref",
                &format!("refs/onebrain/checkpoints/{label}"),
                &commit,
            ],
        )?;
        Ok(commit)
    })();
    let _ = std::fs::remove_file(&index);
    result.map(Some)
}

/// Put the working tree of `repo` back to `checkpoint`: files from the
/// snapshot are rewritten and files created since are removed. The user's
/// index, branch and history are left as they are.
pub fn restore_checkpoint(repo: &Path, checkpoint: &str) -> Result<(), CheckpointError> {
    let tree = format!("{checkpoint}^{{tree}}");
    let wanted = nul_list(&git(
        repo,
        None,
        &["ls-tree", "-r", "-z", "--name-only", &tree],
    )?);
    let present = nul_list(&git(
        repo,
        None,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?);

    let root = std::fs::canonicalize(repo)?;
    for path in present.difference(&wanted) {
        let target = root.join(path);
        if !target.starts_with(&root) || path.split('/').any(|part| part == "..") {
            return Err(CheckpointError::UnsafePath(path.clone()));
        }
        if target.is_file() || target.is_symlink() {
            std::fs::remove_file(&target)?;
        }
    }

    let index = scratch_index(repo, "onebrain-restore.index")?;
    let result = (|| {
        git(repo, Some(&index), &["read-tree", &tree])?;
        git(repo, Some(&index), &["checkout-index", "-a", "-f"])?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&index);
    result
}

/// Tree of the working tree as it is right now, without storing a ref.
fn current_tree(repo: &Path) -> Result<String, CheckpointError> {
    let index = scratch_index(repo, "onebrain-current.index")?;
    let result = (|| {
        git(repo, Some(&index), &["add", "-A", "--", "."])?;
        git(repo, Some(&index), &["write-tree"])
    })();
    let _ = std::fs::remove_file(&index);
    result
}

/// Paths whose content differs between two trees (renames count as a delete
/// plus an add).
fn changed_paths(repo: &Path, from: &str, to: &str) -> Result<HashSet<String>, CheckpointError> {
    Ok(nul_list(&git(
        repo,
        None,
        &[
            "diff-tree",
            "-r",
            "-z",
            "--no-renames",
            "--name-only",
            from,
            to,
        ],
    )?))
}

/// Undo one run: put back only the files the run changed between `before`
/// and `after`. If the user (or another agent) edited one of those files
/// after the run, nothing is written and the conflicting paths are returned,
/// unless `force` is set. Files the run did not touch are never written.
pub fn undo_run(
    repo: &Path,
    before: &str,
    after: &str,
    force: bool,
) -> Result<Vec<String>, CheckpointError> {
    let before_tree = format!("{before}^{{tree}}");
    let after_tree = format!("{after}^{{tree}}");
    let touched = changed_paths(repo, &before_tree, &after_tree)?;
    if touched.is_empty() {
        return Ok(Vec::new());
    }
    let now = current_tree(repo)?;
    let mut conflicts: Vec<String> = changed_paths(repo, &after_tree, &now)?
        .intersection(&touched)
        .cloned()
        .collect();
    conflicts.sort();
    if !conflicts.is_empty() && !force {
        return Err(CheckpointError::Conflict(conflicts));
    }

    let in_before = nul_list(&git(
        repo,
        None,
        &["ls-tree", "-r", "-z", "--name-only", &before_tree],
    )?);
    let root = std::fs::canonicalize(repo)?;
    let mut rewrite = Vec::new();
    for path in &touched {
        if path.split('/').any(|part| part == ".." || part.is_empty()) {
            return Err(CheckpointError::UnsafePath(path.clone()));
        }
        if in_before.contains(path) {
            rewrite.push(path.as_str());
        } else {
            let target = root.join(path);
            if target.is_file() || target.is_symlink() {
                std::fs::remove_file(&target)?;
            }
        }
    }
    if !rewrite.is_empty() {
        let index = scratch_index(repo, "onebrain-undo.index")?;
        let result: Result<(), CheckpointError> = (|| {
            git(repo, Some(&index), &["read-tree", &before_tree])?;
            let mut input = Vec::new();
            for path in &rewrite {
                input.extend_from_slice(path.as_bytes());
                input.push(0);
            }
            git_stdin(
                repo,
                Some(&index),
                &["checkout-index", "-f", "-z", "--stdin"],
                &input,
            )?;
            Ok(())
        })();
        let _ = std::fs::remove_file(&index);
        result?;
    }
    let mut restored: Vec<String> = touched.into_iter().collect();
    restored.sort();
    Ok(restored)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        git(path, None, &["init", "-q", "-b", "main"]).unwrap();
        fs::write(path.join("a.txt"), "original\n").unwrap();
        fs::write(path.join(".gitignore"), "ignored.log\n").unwrap();
        git(path, None, &["add", "-A"]).unwrap();
        git(path, None, &["commit", "-q", "-m", "init"]).unwrap();
        dir
    }

    #[test]
    fn restore_brings_back_files_and_removes_new_ones_without_touching_history() {
        let dir = repo();
        let path = dir.path();
        fs::write(path.join("a.txt"), "edited before run\n").unwrap();
        fs::write(path.join("b.txt"), "new untracked\n").unwrap();
        fs::write(path.join("ignored.log"), "keep me\n").unwrap();
        let head_before = git(path, None, &["rev-parse", "HEAD"]).unwrap();
        let staged_before = git(path, None, &["diff", "--cached", "--name-only"]).unwrap();

        let checkpoint = create_checkpoint(path, "run-1").unwrap().expect("git repo");

        // The agent run changes things.
        fs::write(path.join("a.txt"), "broken by agent\n").unwrap();
        fs::remove_file(path.join("b.txt")).unwrap();
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/c.rs"), "fn added() {}\n").unwrap();

        restore_checkpoint(path, &checkpoint).unwrap();

        assert_eq!(
            fs::read_to_string(path.join("a.txt")).unwrap(),
            "edited before run\n"
        );
        assert_eq!(
            fs::read_to_string(path.join("b.txt")).unwrap(),
            "new untracked\n"
        );
        assert!(
            !path.join("src/c.rs").exists(),
            "files created by the run are removed"
        );
        assert_eq!(
            fs::read_to_string(path.join("ignored.log")).unwrap(),
            "keep me\n"
        );
        assert_eq!(
            git(path, None, &["rev-parse", "HEAD"]).unwrap(),
            head_before
        );
        assert_eq!(
            git(path, None, &["diff", "--cached", "--name-only"]).unwrap(),
            staged_before
        );
        assert_eq!(
            git(
                path,
                None,
                &["rev-parse", "refs/onebrain/checkpoints/run-1"]
            )
            .unwrap(),
            checkpoint
        );
    }

    #[test]
    fn folders_outside_git_are_skipped_and_bad_labels_rejected() {
        let dir = tempfile::tempdir().unwrap();
        assert!(create_checkpoint(dir.path(), "run-1").unwrap().is_none());
        assert!(matches!(
            create_checkpoint(dir.path(), "../escape"),
            Err(CheckpointError::InvalidLabel(_))
        ));
    }

    #[test]
    fn works_in_a_repository_without_commits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        git(path, None, &["init", "-q"]).unwrap();
        fs::write(path.join("first.txt"), "one\n").unwrap();
        let checkpoint = create_checkpoint(path, "first").unwrap().unwrap();
        fs::write(path.join("first.txt"), "two\n").unwrap();
        restore_checkpoint(path, &checkpoint).unwrap();
        assert_eq!(fs::read_to_string(path.join("first.txt")).unwrap(), "one\n");
    }

    #[test]
    fn undo_only_touches_files_the_run_changed() {
        let dir = repo();
        let path = dir.path();
        fs::write(path.join("other.txt"), "user file\n").unwrap();
        let before = create_checkpoint(path, "r1").unwrap().unwrap();
        fs::write(path.join("a.txt"), "agent edit\n").unwrap();
        fs::write(path.join("new.rs"), "fn x() {}\n").unwrap();
        let after = create_checkpoint(path, "r1-after").unwrap().unwrap();
        // The user keeps working on a file the run never touched.
        fs::write(path.join("other.txt"), "user kept typing\n").unwrap();

        let restored = undo_run(path, &before, &after, false).unwrap();

        assert_eq!(restored, vec!["a.txt".to_string(), "new.rs".to_string()]);
        assert_eq!(
            fs::read_to_string(path.join("a.txt")).unwrap(),
            "original\n"
        );
        assert!(!path.join("new.rs").exists());
        assert_eq!(
            fs::read_to_string(path.join("other.txt")).unwrap(),
            "user kept typing\n"
        );
    }

    #[test]
    fn undo_reports_files_edited_after_the_run() {
        let dir = repo();
        let path = dir.path();
        let before = create_checkpoint(path, "r2").unwrap().unwrap();
        fs::write(path.join("a.txt"), "agent edit\n").unwrap();
        let after = create_checkpoint(path, "r2-after").unwrap().unwrap();
        fs::write(path.join("a.txt"), "user fixed it by hand\n").unwrap();

        match undo_run(path, &before, &after, false) {
            Err(CheckpointError::Conflict(paths)) => assert_eq!(paths, vec!["a.txt".to_string()]),
            other => panic!("expected a conflict, got {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(path.join("a.txt")).unwrap(),
            "user fixed it by hand\n"
        );

        undo_run(path, &before, &after, true).unwrap();
        assert_eq!(
            fs::read_to_string(path.join("a.txt")).unwrap(),
            "original\n"
        );
    }
}
