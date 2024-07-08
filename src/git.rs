//! This module handles git operations.

use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};
use log::info;
use std::path::Path;

/// Generates a git diff for the repository at the provided path
///
/// # Arguments
///
/// * `repo_path` - A reference to the path of the git repository
///
/// # Returns
///
/// * `Result<String, git2::Error>` - The generated git diff as a string or an error
pub fn get_git_diff(repo_path: &Path) -> Result<String> {
    info!("Opening repository at path: {:?}", repo_path);
    let repo = Repository::open(repo_path).context("Failed to open repository")?;
    let head = repo.head().context("Failed to get repository head")?;
    let head_tree = head.peel_to_tree().context("Failed to peel to tree")?;

    let diff = repo
        .diff_tree_to_index(
            Some(&head_tree),
            None,
            Some(DiffOptions::new().ignore_whitespace(true)),
        )
        .context("Failed to generate diff")?;

    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.extend_from_slice(line.content());
        true
    })
    .context("Failed to print diff")?;

    info!("Generated git diff successfully");
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
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
            commit.summary().unwrap_or("No commit message")
        ));
    }

    info!("Retrieved git log successfully");
    Ok(log_text)
}

/// Checks if a local branch exists in the given repository
///
/// # Arguments
///
/// * `repo` - A reference to the `Repository` where the branch should be checked
/// * `branch_name` - A string slice that holds the name of the branch to check
///
/// # Returns
///
/// * `bool` - `true` if the branch exists, `false` otherwise
fn branch_exists(repo: &Repository, branch_name: &str) -> bool {
    match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(_) => true,
        Err(_) => false,
    }
}
