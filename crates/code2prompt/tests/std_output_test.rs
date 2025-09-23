//! Standard output tests for code2prompt
//!
//! This module tests stdout functionality, output redirection,
//! and various output modes using rstest fixtures.

mod common;

use assert_cmd::Command;
use common::*;
use log::debug;
use predicates::prelude::*;
use predicates::str::contains;
use rstest::*;

/// Test default stdout behavior (auto-quiet when non-terminal)
#[rstest]
fn test_stdout_output_default(stdout_test_env: StdoutTestEnv) {
    // When stdout is captured by the test harness (non-terminal), auto-quiet suppresses status messages.
    // Ensure the command succeeds and does not print status messages.
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .assert()
        .success()
        .stdout(contains("Token count:").not())
        .stdout(contains("Copied to clipboard successfully").not());

    debug!("✓ Default stdout output test passed");
}

/// Test various stdout output configurations
#[rstest]
#[case("explicit_dash", vec!["-O", "-", "--no-clipboard"], vec!["test.py", "print('Hello, World!')", "README.md", "# Test Project"], vec![], true)]
#[case("long_form", vec!["--output-file", "-", "--no-clipboard"], vec!["test.py", "print('Hello, World!')", "README.md", "# Test Project"], vec![], true)]
#[case("quiet_mode", vec!["--quiet", "-O", "-", "--no-clipboard"], vec!["test.py", "print('Hello, World!')"], vec!["✓"], true)]
fn test_stdout_configurations(
    stdout_test_env: StdoutTestEnv,
    #[case] test_name: &str,
    #[case] args: Vec<&str>,
    #[case] should_contain: Vec<&str>,
    #[case] should_not_contain: Vec<&str>,
    #[case] should_succeed: bool,
) {
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path());

    for arg in args {
        cmd.arg(arg);
    }

    let assertion = cmd.assert();

    if should_succeed {
        let assertion = assertion.success();

        // Check content that should be present
        let mut assertion = assertion;
        for content in should_contain {
            assertion = assertion.stdout(contains(content));
        }

        // Check content that should not be present
        for content in should_not_contain {
            assertion = assertion.stdout(contains(content).not());
        }

        debug!("✓ {} test passed", test_name);
    } else {
        assertion.failure();
        debug!("✓ {} test passed (correctly failed)", test_name);
    }
}

/// Test conflicting output options (should fail)
#[rstest]
fn test_conflicting_output_options_should_fail(stdout_test_env: StdoutTestEnv) {
    // Test: Using both default stdout and explicit -O - should fail
    // This is a logical conflict - you can't output to stdout in two different ways
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("-")
        .arg("-O")
        .arg("-")
        .arg("--no-clipboard")
        .assert()
        .failure();

    debug!("✓ Conflicting output options test passed (correctly failed)");
}

/// Test output file vs stdout conflict (should fail)
#[rstest]
fn test_output_file_vs_stdout_conflict(stdout_test_env: StdoutTestEnv) {
    let output_file = stdout_test_env.dir.path().join("output.txt");

    // Test: Using both output file and stdout should fail
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap())
        .arg("-O")
        .arg("-")
        .arg("--no-clipboard")
        .assert()
        .failure()
        .stderr(
            contains("cannot be used multiple times")
                .or(contains("conflict"))
                .or(contains("mutually exclusive")),
        );

    debug!("✓ Output file vs stdout conflict test passed (correctly failed)");
}

/// Test that --no-clipboard requires --output-file
#[rstest]
fn test_no_clipboard_requires_output_file(stdout_test_env: StdoutTestEnv) {
    // Test: When --no-clipboard is used, --output-file is required
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--no-clipboard")
        .assert()
        .failure()
        .stderr(contains(
            "--output-file is required when --no-clipboard is used",
        ));

    debug!("✓ No clipboard requires output file test passed");
}

/// Test stdout with different output formats
#[rstest]
#[case("json", "{", "\"files\"")]
#[case("xml", "<", ">")]
#[case("markdown", "Source Tree:", "```")]
fn test_stdout_with_different_formats(
    stdout_test_env: StdoutTestEnv,
    #[case] format: &str,
    #[case] expected_start: &str,
    #[case] expected_content: &str,
) {
    // Test: Stdout should work with different output formats
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--output-format")
        .arg(format)
        .arg("-O")
        .arg("-")
        .arg("--no-clipboard")
        .assert()
        .success()
        .stdout(contains(expected_start))
        .stdout(contains(expected_content))
        .stdout(contains("test.py"));

    debug!("✓ Stdout with {} format test passed", format);
}

/// Test that fixture creates proper test environment
#[rstest]
fn test_stdout_fixture_setup(stdout_test_env: StdoutTestEnv) {
    // Verify that the fixture created the expected files
    let test_files = vec!["test.py", "README.md", "config.json"];

    for file in test_files {
        let file_path = stdout_test_env.dir.path().join(file);
        assert!(file_path.exists(), "Test file {} should exist", file);
    }

    debug!("✓ Stdout fixture setup test passed");
}
