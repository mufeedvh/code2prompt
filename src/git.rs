use std::path::Path;
use git2::{DiffOptions, Repository};
use anyhow::Result;
use log::{info, error};

/// Generates a git diff for the repository at the provided path
pub fn get_git_diff(repo_path: &Path) -> Result<String, git2::Error> {
    info!("Opening repository at path: {:?}", repo_path);
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            error!("Failed to open repository: {}", e);
            return Err(e);
        }
    };

    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            error!("Failed to get repository head: {}", e);
            return Err(e);
        }
    };

    let head_tree = match head.peel_to_tree() {
        Ok(tree) => tree,
        Err(e) => {
            error!("Failed to peel to tree: {}", e);
            return Err(e);
        }
    };

    let diff = match repo.diff_tree_to_index(
        Some(&head_tree),
        None,
        Some(DiffOptions::new().ignore_whitespace(true)),
    ) {
        Ok(diff) => diff,
        Err(e) => {
            error!("Failed to generate diff: {}", e);
            return Err(e);
        }
    };

    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.extend_from_slice(line.content());
        true
    })?;

    info!("Generated git diff successfully");
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}
