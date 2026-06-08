//! This module handles git operations.

use crate::configuration::DiffMode;
use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};
use log::debug;
use std::path::Path;

/// Generates a git diff for the repository, selecting which changes to show.
///
/// * `Staged`   — HEAD vs index (what `git diff --cached` shows)
/// * `Unstaged` — index vs working tree (what `git diff` shows)
/// * `All`      — HEAD vs working tree, staged and unstaged together
pub fn get_git_diff(repo_path: &Path, mode: DiffMode) -> Result<String> {
    debug!("Opening repository at path: {:?}", repo_path);
    let repo = Repository::open(repo_path).context("Failed to open repository")?;

    let head_tree = repo
        .head()
        .context("Failed to get repository head")?
        .peel_to_tree()
        .context("Failed to peel to tree")?;

    let mut opts = DiffOptions::new();

    let statuses = repo.statuses(None).context("status")?;
    for e in statuses.iter() {
        debug!("git2 sees: {:?} status={:?}", e.path(), e.status());
    }

    let diff = match mode {
        DiffMode::Staged => {
            let mut index = repo.index().context("Failed to read index")?;
            index
                .read(true)
                .context("Failed to refresh index from disk")?;
            repo.diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut opts))
                .context("Failed to diff HEAD to index (staged)")?
        }
        DiffMode::Unstaged => {
            // Match plain `git diff`: working tree vs index, including intent-to-add
            // (`git add -N`) files, whose content lives in the worktree only.
            opts.include_untracked(false); // git diff excludes pure untracked
            let mut index = repo.index().context("Failed to read index")?;
            index.read(true).context("Failed to refresh index")?; // re-read from disk, avoid stale state
            repo.diff_index_to_workdir(Some(&index), Some(&mut opts))
                .context("Failed to diff index to working tree (unstaged)")?
        }
        DiffMode::All => {
            opts.include_untracked(true).recurse_untracked_dirs(true);
            repo.diff_tree_to_workdir(Some(&head_tree), Some(&mut opts))
                .context("Failed to diff HEAD to working tree (all uncommitted)")?
        }
    };

    let mut buf = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        buf.extend_from_slice(line.content());
        true
    })
    .context("Failed to print diff")?;
    debug!(
        "diff produced {} deltas, {} bytes",
        diff.deltas().len(),
        buf.len()
    );

    let text = String::from_utf8_lossy(&buf).into_owned();
    if text.trim().is_empty() {
        // Honest per-mode empty message instead of the old HEAD/index-only one.
        let what = match mode {
            DiffMode::Staged => "staged changes (HEAD vs index)",
            DiffMode::Unstaged => "unstaged changes (index vs working tree)",
            DiffMode::All => "uncommitted changes",
        };
        return Ok(format!("no {what}"));
    }
    Ok(text)
}

/// Generates a git diff between two branches for the repository at the provided path
///
/// # Arguments
///
/// * `repo_path` - A reference to the path of the git repository
/// * `branch1` - The name of the first branch
/// * `branch2` - The name of the second branch
///
/// # Returns
///
/// * `Result<String, git2::Error>` - The generated git diff as a string or an error
pub fn get_git_diff_between_branches(
    repo_path: &Path,
    branch1: &str,
    branch2: &str,
) -> Result<String> {
    info!("Opening repository at path: {:?}", repo_path);
    let repo = Repository::open(repo_path).context("Failed to open repository")?;

    for branch in [branch1, branch2].iter() {
        if !branch_exists(&repo, branch) {
            return Err(anyhow::anyhow!("Branch {} doesn't exist!", branch));
        }
    }

    let branch1_commit = repo.revparse_single(branch1)?.peel_to_commit()?;
    let branch2_commit = repo.revparse_single(branch2)?.peel_to_commit()?;

    let branch1_tree = branch1_commit.tree()?;
    let branch2_tree = branch2_commit.tree()?;

    let diff = repo
        .diff_tree_to_tree(
            Some(&branch1_tree),
            Some(&branch2_tree),
            Some(DiffOptions::new().ignore_whitespace(true)),
        )
        .context("Failed to generate diff between branches")?;

    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.extend_from_slice(line.content());
        true
    })
    .context("Failed to print diff")?;

    info!("Generated git diff between branches successfully");
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}

/// Retrieves the git log between two branches for the repository at the provided path
///
/// # Arguments
///
/// * `repo_path` - A reference to the path of the git repository
/// * `branch1` - The name of the first branch (e.g., "master")
/// * `branch2` - The name of the second branch (e.g., "migrate-manifest-v3")
///
/// # Returns
///
/// * `Result<String, git2::Error>` - The git log as a string or an error
pub fn get_git_log(repo_path: &Path, branch1: &str, branch2: &str) -> Result<String> {
    info!("Opening repository at path: {:?}", repo_path);
    let repo = Repository::open(repo_path).context("Failed to open repository")?;

    for branch in [branch1, branch2].iter() {
        if !branch_exists(&repo, branch) {
            return Err(anyhow::anyhow!("Branch {} doesn't exist!", branch));
        }
    }

    let branch1_commit = repo.revparse_single(branch1)?.peel_to_commit()?;
    let branch2_commit = repo.revparse_single(branch2)?.peel_to_commit()?;

    let mut revwalk = repo.revwalk().context("Failed to create revwalk")?;
    revwalk
        .push(branch2_commit.id())
        .context("Failed to push branch2 commit to revwalk")?;
    revwalk
        .hide(branch1_commit.id())
        .context("Failed to hide branch1 commit from revwalk")?;
    revwalk.set_sorting(git2::Sort::REVERSE)?;

    let mut log_text = String::new();
    for oid in revwalk {
        let oid = oid.context("Failed to get OID from revwalk")?;
        let commit = repo.find_commit(oid).context("Failed to find commit")?;
        log_text.push_str(&format!(
            "{} - {}\n",
            &commit.id().to_string()[..7],
            commit
                .summary()
                .ok()
                .flatten()
                .unwrap_or("No commit message")
        ));
    }

    info!("Retrieved git log successfully");
    Ok(log_text)
}

/// Checks if a git reference exists in the given repository
///
/// This function can validate any git reference including:
/// - Local and remote branch names
/// - Commit hashes (full or abbreviated)
/// - Tags
/// - Any reference that git rev-parse can resolve
///
/// # Arguments
///
/// * `repo` - A reference to the `Repository` where the reference should be checked
/// * `branch_name` - A string slice that holds the name of the reference to check
///
/// # Returns
///
/// * `bool` - `true` if the reference exists, `false` otherwise
fn branch_exists(repo: &Repository, branch_name: &str) -> bool {
    repo.revparse_single(branch_name).is_ok()
}
