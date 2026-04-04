/// Git integration for gathering repository history, diffs, and hot files.
/// Inspired by repomix's git features and aider's change-frequency ranking.

use std::process::Command;
use std::path::Path;
use tracing::warn;

use crate::models::{GitSummary, CommitInfo};

/// Gather git summary for a repository: recent commits, diff stats, and hot files.
pub fn gather_git_summary(repo_path: &str, max_commits: usize) -> Option<GitSummary> {
    let path = Path::new(repo_path);
    if !path.join(".git").exists() {
        return None;
    }

    let recent_commits = get_recent_commits(repo_path, max_commits);
    let diff_stat = get_diff_stat(repo_path);
    let hot_files = get_hot_files(repo_path, 100);

    Some(GitSummary {
        recent_commits,
        diff_stat,
        hot_files,
    })
}

/// Get recent git commits with messages, dates, and changed files.
fn get_recent_commits(repo_path: &str, count: usize) -> Vec<CommitInfo> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-n{}", count),
            "--format=%H|%as|%s",
            "--name-only",
        ])
        .current_dir(repo_path)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        Ok(o) => {
            warn!("git log failed: {}", String::from_utf8_lossy(&o.stderr));
            return Vec::new();
        }
        Err(e) => {
            warn!("Failed to run git log: {}", e);
            return Vec::new();
        }
    };

    let mut commits = Vec::new();
    let mut current: Option<CommitInfo> = None;

    for line in output.lines() {
        if line.contains('|') {
            // This is a commit header line: hash|date|message
            if let Some(commit) = current.take() {
                commits.push(commit);
            }
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() == 3 {
                current = Some(CommitInfo {
                    hash: parts[0][..8.min(parts[0].len())].to_string(),
                    date: parts[1].to_string(),
                    message: parts[2].to_string(),
                    files: Vec::new(),
                });
            }
        } else if !line.trim().is_empty() {
            // This is a file path
            if let Some(ref mut commit) = current {
                commit.files.push(line.trim().to_string());
            }
        }
    }
    if let Some(commit) = current {
        commits.push(commit);
    }

    commits
}

/// Get working tree + staged diff statistics.
pub fn get_diff_stat(repo_path: &str) -> String {
    let mut result = String::new();

    // Staged changes
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            let staged = String::from_utf8_lossy(&output.stdout);
            if !staged.trim().is_empty() {
                result.push_str("=== Staged Changes ===\n");
                result.push_str(&staged);
                result.push('\n');
            }
        }
    }

    // Working tree changes
    if let Ok(output) = Command::new("git")
        .args(["diff", "--stat"])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            let working = String::from_utf8_lossy(&output.stdout);
            if !working.trim().is_empty() {
                result.push_str("=== Working Tree Changes ===\n");
                result.push_str(&working);
            }
        }
    }

    result
}

/// Get full diff content (staged + working tree).
pub fn get_full_diff(repo_path: &str) -> String {
    let mut result = String::new();

    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached"])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            let diff = String::from_utf8_lossy(&output.stdout);
            if !diff.trim().is_empty() {
                result.push_str(&diff);
                result.push('\n');
            }
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["diff"])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            result.push_str(&String::from_utf8_lossy(&output.stdout));
        }
    }

    result
}

/// Get files sorted by change frequency (most changed first).
/// This is inspired by aider's graph-ranking and repomix's --git-sort-by-changes.
fn get_hot_files(repo_path: &str, max_commits: usize) -> Vec<(String, usize)> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-n{}", max_commits),
            "--format=format:",
            "--name-only",
        ])
        .current_dir(repo_path)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    let mut file_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.is_empty() {
            *file_counts.entry(line.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<(String, usize)> = file_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(30);
    sorted
}
