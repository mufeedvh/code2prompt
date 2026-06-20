//! The git-narrative templates (commit / changeset / PR) must scope their
//! source tree to the files involved in the change, not the whole repo.
//! Guards the changed-files-tree routing in build_spec.
//!
//! Pipeline-only: under the legacy path the whole tree still renders, so the
//! "unchanged file absent" assertion only holds with the feature on. The gate
//! comes off at Step 6 when the pipeline is the sole path.

use git2::{Repository, Signature};
use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Commit a.txt + untouched.txt, then (working tree) modify a.txt and add an
/// untracked b.txt. untouched.txt is committed and never touched again.
fn repo_with_uncommitted_changes() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("T", "t@e.com").unwrap();

    fs::write(dir.path().join("a.txt"), "original\n").unwrap();
    fs::write(dir.path().join("untouched.txt"), "stable\n").unwrap();
    let mut idx = repo.index().unwrap();
    idx.add_path(Path::new("a.txt")).unwrap();
    idx.add_path(Path::new("untouched.txt")).unwrap();
    idx.write().unwrap();
    let tree = repo.find_tree(idx.write_tree().unwrap()).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();

    fs::write(dir.path().join("a.txt"), "MODIFIED\n").unwrap();
    fs::write(dir.path().join("b.txt"), "new file\n").unwrap();
    dir
}

#[test]
fn changeset_tree_lists_only_changed_files() {
    let dir = repo_with_uncommitted_changes();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.current_dir(dir.path())
        .arg(".")
        // --diff + all auto-selects write-git-changeset-commits and includes
        // the untracked b.txt (unstaged mode would omit untracked, like `git diff`)
        .args(["--diff", "--diff-mode", "all"])
        .args(["-O", "-", "--no-clipboard", "--quiet"])
        .assert()
        .success()
        // changed files appear in the source tree
        .stdout(contains("a.txt"))
        .stdout(contains("b.txt"))
        // the unchanged committed file must NOT be in the tree
        .stdout(contains("untouched.txt").not())
        // the diff still renders as chrome
        .stdout(contains("MODIFIED"));
}
