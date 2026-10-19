//! Git repository integration for diff extraction and branch operations.
//! 
//! This module provides core git functionality for code2prompt, including:
//! - Generating diffs between HEAD and index (staged changes)
//! - Comparing branches for diff and log extraction
//! - Repository validation and reference checking

use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};
use log::info;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Synthesize a diff-line sigil (`+`/`-`/` `) for added/removed/context lines.
///
/// `git2`'s `DiffFormat::Patch` splits the sigil from the line content:
/// - `origin()` returns `'+'` / `'-'` / `' '` for added/removed/context lines
/// - `content()` returns ONLY the raw line text WITHOUT the sigil
/// - file headers (`'F'`), hunk headers (`'H'`), binary notices (`'B'`) and the
///   end-of-file markers (`'='/'>'/'<'`) already contain their full text in
///   `content()`, so they get no synthesized sigil.
///
/// `content()` is appended unconditionally, so binary notices and the
/// `\ No newline at end of file` markers are preserved rather than dropped.
pub fn write_diff_line(buf: &mut Vec<u8>, origin: char, content: &[u8]) {
    match origin {
        '+' => buf.push(b'+'),
        '-' => buf.push(b'-'),
        ' ' => buf.push(b' '),
        'F' | 'H' | 'B' | '=' | '>' | '<' => {} // full text already in content()
        _ => {}
    }
    buf.extend_from_slice(content);
}

/// Extract git diff for staged changes in repository.
/// 
/// Generates diff between HEAD and index to show staged changes, with additional
/// notification if unstaged changes are present. This is the primary git integration
/// point for code2prompt's diff functionality.
///
/// # Arguments
///
/// * `repo_path` - Path to git repository
///
/// # Returns
///
/// * `Result<String>` - Git diff output or "no diff between HEAD and index" if no changes
///
/// # Errors  
/// 
/// Returns error if:
/// - Repository is invalid or cannot be opened
/// - HEAD reference is missing or corrupt
/// - Git operations fail during diff generation
pub fn get_git_diff(repo_path: &Path) -> Result<String> {
    info!("Opening repository at path: {:?}", repo_path);
    let repo = Repository::open(repo_path).context("Failed to open repository")?;

    let head = repo.head().context("Failed to get repository head")?;
    let head_tree = head.peel_to_tree().context("Failed to peel to tree")?;

    // Generate diff for staged changes (HEAD vs. index)
    let staged_diff = repo
        .diff_tree_to_index(
            Some(&head_tree),
            None,
            Some(DiffOptions::new().ignore_whitespace(true)),
        )
        .context("Failed to generate diff for staged changes")?;

    let mut staged_diff_text = Vec::new();
    staged_diff
        .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            write_diff_line(&mut staged_diff_text, line.origin(), line.content());
            true
        })
        .context("Failed to print staged diff")?;

    let staged_diff_output = String::from_utf8_lossy(&staged_diff_text).into_owned();

    // If there is no staged diff, return a message indicating so.
    if staged_diff_output.trim().is_empty() {
        return Ok("no diff between HEAD and index".to_string());
    }

    // Generate diff for unstaged changes (index vs. working directory)
    let unstaged_diff = repo
        .diff_index_to_workdir(None, Some(DiffOptions::new().ignore_whitespace(true)))
        .context("Failed to generate diff for unstaged changes")?;

    let mut unstaged_diff_text = Vec::new();
    unstaged_diff
        .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            write_diff_line(&mut unstaged_diff_text, line.origin(), line.content());
            true
        })
        .context("Failed to print unstaged diff")?;

    let unstaged_diff_output = String::from_utf8_lossy(&unstaged_diff_text).into_owned();

    let mut output = staged_diff_output;
    if !unstaged_diff_output.trim().is_empty() {
        output.push_str("\nNote: Some changes are not staged.");
    }

    info!("Generated git diff successfully");
    Ok(output)
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
        write_diff_line(&mut diff_text, line.origin(), line.content());
        true
    })
    .context("Failed to print diff")?;

    info!("Generated git diff between branches successfully");
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}

/// Computes the set of relative file paths that changed between two branches.
///
/// This performs the same tree-to-tree diff as [`get_git_diff_between_branches`],
/// but instead of rendering a textual patch it collects the paths touched by
/// each delta (both sides, so a rename contributes its old and new path).
/// This set is used to prune the source tree and file content down to only
/// the files that actually changed when `--git-diff-branch` is active.
///
/// # Arguments
///
/// * `repo_path` - A reference to the path of the git repository
/// * `branch1` - The name of the first branch
/// * `branch2` - The name of the second branch
///
/// # Returns
///
/// * `Result<HashSet<PathBuf>>` - The relative paths that changed between the two branches
pub fn get_git_diff_file_paths(
    repo_path: &Path,
    branch1: &str,
    branch2: &str,
) -> Result<HashSet<PathBuf>> {
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

    let mut paths = HashSet::new();
    for delta in diff.deltas() {
        if let Some(path) = delta.old_file().path() {
            paths.insert(path.to_path_buf());
        }
        if let Some(path) = delta.new_file().path() {
            paths.insert(path.to_path_buf());
        }
    }

    info!(
        "Collected {} changed file path(s) between branches",
        paths.len()
    );
    Ok(paths)
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
        let summary = commit.summary().ok().flatten().unwrap_or("No commit message");
        log_text.push_str(&format!(
            "{} - {}\n",
            &commit.id().to_string()[..7],
            summary
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
