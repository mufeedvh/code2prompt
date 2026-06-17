//! This module handles git operations.

use crate::configuration::{DiffMode, DiffShaContent};
use anyhow::{Context, Result};
use git2::{Delta, DiffOptions, Oid, Patch, Repository};
use log::debug;
use serde::Serialize;
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
            opts.include_untracked(true)
                .recurse_untracked_dirs(true)
                .show_untracked_content(true);
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
    debug!("Opening repository at path: {:?}", repo_path);
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

    debug!("Generated git diff between branches successfully");
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}

/// One file changed between two refs, with selected content.
#[derive(Debug, Clone, Serialize)]
pub struct ChangedFile {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>, // Some only on rename
    pub status: &'static str, // added | deleted | modified | renamed | copied | other
    pub binary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
}

enum BlobRead {
    Text(String),
    Binary,
    Skip, // absent (zero oid) or over the size cap
}

fn read_blob(repo: &Repository, oid: Oid, max_bytes: usize) -> BlobRead {
    if oid.is_zero() {
        return BlobRead::Skip;
    }
    let Ok(blob) = repo.find_blob(oid) else {
        return BlobRead::Skip;
    };
    if blob.is_binary() {
        return BlobRead::Binary;
    }
    if max_bytes != 0 && blob.content().len() > max_bytes {
        return BlobRead::Skip;
    }
    BlobRead::Text(String::from_utf8_lossy(blob.content()).into_owned())
}

/// Files changed between two refs, each with the content selected by `mode`.
///
/// `max_file_bytes` caps per-file content (0 = no cap); oversized or binary
/// files are reported with content left `None`.
pub fn get_changed_files_with_contents(
    repo_path: &Path,
    ref1: &str,
    ref2: &str,
    mode: DiffShaContent,
    max_file_bytes: usize,
) -> Result<Vec<ChangedFile>> {
    debug!("Opening repository at path: {:?}", repo_path);
    debug!(
        "diff-shas: repo={:?} ref1={} ref2={}",
        repo_path, ref1, ref2
    );
    let repo = Repository::open(repo_path).context("Failed to open repository")?;

    let tree1 = repo
        .revparse_single(ref1)
        .with_context(|| format!("Cannot resolve revision {ref1}"))?
        .peel_to_tree()
        .with_context(|| format!("{ref1} does not point to a tree"))?;
    let tree2 = repo
        .revparse_single(ref2)
        .with_context(|| format!("Cannot resolve revision {ref2}"))?
        .peel_to_tree()
        .with_context(|| format!("{ref2} does not point to a tree"))?;

    // Whole-blob reads here, so do NOT ignore_whitespace (unlike the branch diff).
    let mut diff = repo
        .diff_tree_to_tree(Some(&tree1), Some(&tree2), Some(&mut DiffOptions::new()))
        .context("Failed to diff the two trees")?;
    diff.find_similar(None).context("Rename detection failed")?;

    debug!("diff-shas: resolved {} deltas", diff.deltas().len());

    let want_patch = matches!(
        mode,
        DiffShaContent::Patch | DiffShaContent::AfterPatch | DiffShaContent::FullPatch
    );

    let mut out = Vec::with_capacity(diff.deltas().len());
    for (idx, delta) in diff.deltas().enumerate() {
        let is_add = delta.status() == Delta::Added;
        let status = match delta.status() {
            Delta::Added => "added",
            Delta::Deleted => "deleted",
            Delta::Modified => "modified",
            Delta::Renamed => "renamed",
            Delta::Copied => "copied",
            _ => "other",
        };

        let new_path = delta
            .new_file()
            .path()
            .map(|p| p.to_string_lossy().into_owned());
        let old_path = delta
            .old_file()
            .path()
            .map(|p| p.to_string_lossy().into_owned());
        let path = new_path
            .clone()
            .or_else(|| old_path.clone())
            .unwrap_or_else(|| "<unknown>".to_string());

        let mut binary = delta.old_file().is_binary() || delta.new_file().is_binary();

        // What to read depends on the mode (and, in Patch mode, the status).
        let (do_before, do_after) = match mode {
            DiffShaContent::Full | DiffShaContent::FullPatch => (true, true),
            DiffShaContent::AfterPatch => (false, true), // full current file, no before
            DiffShaContent::Patch => (false, is_add),    // lean: patch carries the rest
        };

        let before = if do_before && !binary {
            match read_blob(&repo, delta.old_file().id(), max_file_bytes) {
                BlobRead::Text(t) => Some(t),
                BlobRead::Binary => {
                    binary = true;
                    None
                }
                BlobRead::Skip => None,
            }
        } else {
            None
        };
        let after = if do_after && !binary {
            match read_blob(&repo, delta.new_file().id(), max_file_bytes) {
                BlobRead::Text(t) => Some(t),
                BlobRead::Binary => {
                    binary = true;
                    None
                }
                BlobRead::Skip => None,
            }
        } else {
            None
        };

        // No patch for adds (redundant with `after`) or binaries.
        let patch = if want_patch && !is_add && !binary {
            Patch::from_diff(&diff, idx)
                .ok()
                .flatten()
                .and_then(|mut p| p.to_buf().ok())
                .map(|b| String::from_utf8_lossy(&b).into_owned())
        } else {
            None
        };

        out.push(ChangedFile {
            path,
            old_path: if delta.status() == Delta::Renamed {
                old_path
            } else {
                None
            },
            status,
            binary,
            before,
            after,
            patch,
        });
    }

    debug!(
        "changed-files: {} entries between {} and {}",
        out.len(),
        ref1,
        ref2
    );
    Ok(out)
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
    debug!("Opening repository at path: {:?}", repo_path);
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

    debug!("Retrieved git log successfully");
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
