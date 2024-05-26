use std::path::Path;
use git2::{DiffOptions, Repository};
use anyhow::Result;


/// Generates a git diff for the repository at the provided path
pub fn get_git_diff(repo_path: &Path) -> Result<String, git2::Error> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(
        Some(&head_tree),
        None,
        Some(DiffOptions::new().ignore_whitespace(true)),
    )?;
    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.extend_from_slice(line.content());
        true
    })?;
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}
