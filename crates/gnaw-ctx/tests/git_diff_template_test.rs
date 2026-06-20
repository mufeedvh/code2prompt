//! CLI guard: --diff must render the working-tree diff into the prompt.
//! Passes on both the legacy and `--features pipeline` builds — git_diff is
//! template chrome both paths must supply. This exact field silently
//! regressed once (pipeline RendererConfig.git_diff was hardcoded None).

use git2::{Repository, Signature};
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Commit "original", then overwrite the working tree with "MODIFIED LINE"
/// (unstaged). The string "original" now exists ONLY in the diff's removed
/// line, never in the working-tree file — so finding it proves the diff
/// rendered, not the file body.
fn repo_with_unstaged_change() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("T", "t@e.com").unwrap();

    let file = dir.path().join("a.txt");
    fs::write(&file, "original\n").unwrap();
    let mut idx = repo.index().unwrap();
    idx.add_path(Path::new("a.txt")).unwrap();
    idx.write().unwrap();
    let tree = repo.find_tree(idx.write_tree().unwrap()).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();

    fs::write(&file, "MODIFIED LINE\n").unwrap();
    dir
}

#[test]
fn diff_flag_includes_working_tree_diff() {
    let dir = repo_with_unstaged_change();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.current_dir(dir.path())
        .arg(".")
        .args(["--diff", "--diff-mode", "unstaged"])
        .args(["-O", "-", "--no-clipboard", "--quiet"])
        .assert()
        .success()
        .stdout(contains("MODIFIED LINE")) // the added line
        .stdout(contains("original")); // removed line — only the diff has it
}
