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
