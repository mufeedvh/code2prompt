//! Git integration tests for code2prompt
//!
//! This module tests git-related functionality including gitignore handling
//! and git repository integration using rstest fixtures.

mod common;

use common::fixtures::*;
use common::*;
use log::debug;
use predicates::prelude::*;
use predicates::str::contains;
use rstest::*;

/// Test gitignore functionality - files should be ignored by default
#[rstest]
fn test_gitignore(git_test_env: GitTestEnv) {
    let mut cmd = git_test_env.command();
    cmd.assert().success();

    let output = git_test_env.read_output();
    debug!("Test gitignore output:\n{}", output);

    // Should include files not in gitignore
    assert!(contains("included.txt").eval(&output));
    assert!(contains("Included file").eval(&output));

    // Should exclude files in gitignore
    assert!(contains("ignored.txt").not().eval(&output));
    assert!(contains("Ignored file").not().eval(&output));
}

/// Test --no-ignore flag - should include gitignored files
#[rstest]
fn test_gitignore_no_ignore(git_test_env: GitTestEnv) {
    let mut cmd = git_test_env.command();
    cmd.arg("--no-ignore").assert().success();

    let output = git_test_env.read_output();
    debug!("Test --no-ignore flag output:\n{}", output);

    // Should include all files when ignoring gitignore
    assert!(contains("included.txt").eval(&output));
    assert!(contains("Included file").eval(&output));
    assert!(contains("ignored.txt").eval(&output));
    assert!(contains("Ignored file").eval(&output));
}

/// Test that git repository is properly initialized in fixture
#[rstest]
fn test_git_repo_initialization(git_test_env: GitTestEnv) {
    // Verify that the git repository exists
    let git_dir = git_test_env.dir.path().join(".git");
    assert!(git_dir.exists(), "Git repository should be initialized");
    assert!(git_dir.is_dir(), "Git directory should be a directory");
}

/// Test gitignore with different patterns
#[rstest]
#[case("*.log", "test.log", "Log file content")]
#[case("build/", "build/output.txt", "Build output")]
#[case("*.tmp", "temp.tmp", "Temporary content")]
fn test_gitignore_patterns(
    #[case] pattern: &str,
    #[case] file_path: &str,
    #[case] file_content: &str,
) {
    let env = GitTestEnv::new();

    // Create the test file
    create_temp_file(env.dir.path(), file_path, file_content);

    // Create gitignore with the pattern
    let gitignore_path = env.dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, pattern).expect("Failed to write gitignore");

    let mut cmd = env.command();
    cmd.assert().success();

    let output = env.read_output();
    debug!("Test gitignore pattern '{}' output:\n{}", pattern, output);

    // File should be ignored
    assert!(
        contains(file_content).not().eval(&output),
        "File with pattern '{}' should be ignored",
        pattern
    );

    // Test with --no-ignore
    let mut cmd_no_ignore = env.command();
    cmd_no_ignore.arg("--no-ignore").assert().success();

    let output_no_ignore = env.read_output();
    assert!(
        contains(file_content).eval(&output_no_ignore),
        "File with pattern '{}' should be included with --no-ignore",
        pattern
    );
}
