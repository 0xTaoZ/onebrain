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
}
