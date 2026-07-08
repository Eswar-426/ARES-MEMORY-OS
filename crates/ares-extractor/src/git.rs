//! Git integration — reads commit info and diffs from a local repository.

use ares_core::AresError;
use std::path::Path;
use std::process::Command;
use tracing::debug;

/// Information about a single git commit.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub diff: String,
    pub files_changed: Vec<String>,
}

/// Fetch commit information for a given hash (or HEAD) from a repository.
pub fn get_commit_info(
    repo_path: &Path,
    commit_hash: Option<&str>,
) -> Result<CommitInfo, AresError> {
    let hash_ref = commit_hash.unwrap_or("HEAD");
    debug!(repo = %repo_path.display(), commit = hash_ref, "Fetching commit info");

    // Get the full commit hash
    let hash = run_git(repo_path, &["rev-parse", hash_ref])?;

    // Get the commit message
    let message = run_git(repo_path, &["log", "-1", "--format=%B", &hash])?;

    // Get the author
    let author = run_git(repo_path, &["log", "-1", "--format=%an <%ae>", &hash])?;

    // Get the diff (limited to a reasonable size for LLM processing)
    let diff = run_git(
        repo_path,
        &[
            "diff",
            &format!("{}~1..{}", hash, hash),
            "--stat",
            "-p",
            "--no-color",
        ],
    )?;

    // Get changed files
    let files_output = run_git(
        repo_path,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", &hash],
    )?;
    let files_changed: Vec<String> = files_output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(CommitInfo {
        hash,
        message,
        author,
        diff,
        files_changed,
    })
}

/// Run a git command and return its stdout as a trimmed string.
fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, AresError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| AresError::db(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AresError::db(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_get_commit_info_on_this_repo() {
        // This test runs against the actual ARES repository
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        if !repo_path.join(".git").exists() {
            // Skip if not in a git repo (CI environments)
            return;
        }

        let info = get_commit_info(repo_path, None).unwrap();
        assert!(!info.hash.is_empty());
        assert!(!info.message.is_empty());
        assert!(!info.author.is_empty());
    }
}
