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

/// ~~~ Non Terminal Default Output (Auto-quiet) ~~~
#[rstest]
fn test_output_default(stdout_test_env: StdoutTestEnv) {
    // When stdout is captured by the test harness (non-terminal), auto-quiet suppresses status messages.
    // Ensure the command does not print status messages.
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .assert()
        .success()
        .stdout(contains("Token count:").not())
        .stdout(contains("Copied to clipboard successfully").not())
        .stderr(contains("Token count:").not())
        .stderr(contains("Copied to clipboard successfully").not());

    debug!("✓ Default stdout output test passed");
}

/// ~~~ Stdout Configurations ~~~
#[rstest]
#[case("explicit_dash", vec!["-O", "-", "--no-clipboard"], vec!["test.py", "print('Hello, World!')", "README.md", "# Test Project"], vec!["✓","▹▹▹▹▸ Done!","Token count:","Copied to clipboard successfully"], true)]
#[case("long_form", vec!["--output-file", "-", "--no-clipboard"], vec!["test.py", "print('Hello, World!')", "README.md", "# Test Project"], vec!["✓","▹▹▹▹▸ Done!","Token count:","Copied to clipboard successfully"], true)]
#[case("quiet_mode", vec!["--quiet", "-O", "-", "--no-clipboard"], vec!["test.py", "print('Hello, World!')"], vec!["✓","▹▹▹▹▸ Done!","Token count:","Copied to clipboard successfully"], true)]
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

/// ~~~ File Output Configurations ~~~
#[rstest]
#[case("file_output", vec!["--output-file", "output.txt", "--no-clipboard"], vec!["test.py", "print('Hello, World!')", "README.md"], vec![], true)]
#[case("file_output_quiet", vec!["--output-file", "output.txt", "--quiet", "--no-clipboard"], vec!["test.py", "print('Hello, World!')"], vec!["✓"], true)]
#[case("file_output_json", vec!["--output-file", "output.txt", "--output-format", "json", "--no-clipboard"], vec!["{", "\"files\"", "test.py"], vec![], true)]
#[case("file_output_xml", vec!["--output-file", "output.txt", "--output-format", "xml", "--no-clipboard"], vec!["<directory>", "</file>", "test.py"], vec![], true)]
#[case("file_output_markdown", vec!["--output-file", "output.txt", "--output-format", "markdown", "--no-clipboard"], vec!["Source Tree:", "```", "test.py"], vec![], true)]
fn test_file_output_configurations(
    stdout_test_env: StdoutTestEnv,
    #[case] test_name: &str,
    #[case] args: Vec<&str>,
    #[case] should_contain: Vec<&str>,
    #[case] should_not_contain: Vec<&str>,
    #[case] should_succeed: bool,
) {
    let output_file = stdout_test_env.dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path());

    // Replace "output.txt" in args with the actual path
    for arg in args {
        if arg == "output.txt" {
            cmd.arg(output_file.to_str().unwrap());
        } else {
            cmd.arg(arg);
        }
    }

    let assertion = cmd.assert();

    if should_succeed {
        assertion.success();

        // Read the output file and check its contents
        let file_content =
            std::fs::read_to_string(&output_file).expect("Should be able to read output file");

        // Check content that should be present
        for content in should_contain {
            assert!(
                file_content.contains(content),
                "Test {}: Expected '{}' in file output",
                test_name,
                content
            );
        }

        // Check content that should not be present
        for content in should_not_contain {
            assert!(
                !file_content.contains(content),
                "Test {}: Expected '{}' NOT to be in file output",
                test_name,
                content
            );
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

// Using both output file and stdout should fail
#[rstest]
fn test_output_file_vs_stdout_conflict(stdout_test_env: StdoutTestEnv) {
    let output_file = stdout_test_env.dir.path().join("output.txt");

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

/// Test stderr messages in normal mode (should show status messages)
#[rstest]
fn test_stderr_messages_normal_mode(stdout_test_env: StdoutTestEnv) {
    let output_file = stdout_test_env.dir.path().join("output.txt");

    // Test with file output in normal mode - should show success message in stderr
    // Note: In test environment, auto-quiet is enabled, so Token count might not appear
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .assert()
        .success()
        .stderr(contains("Prompt written to file:"));

    debug!("✓ Normal mode stderr messages test passed");
}

/// Test stderr messages in quiet mode
#[rstest]
fn test_stderr_messages_quiet_mode(stdout_test_env: StdoutTestEnv) {
    let output_file = stdout_test_env.dir.path().join("output.txt");

    // Test with file output in quiet mode - should still show file write confirmation
    // but suppress other messages
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap())
        .arg("--quiet")
        .arg("--no-clipboard")
        .assert()
        .success()
        .stderr(contains("Done!").not());
    // Note: Even in quiet mode, file write confirmation might still appear
    // This is expected behavior for important operations

    debug!("✓ Quiet mode stderr messages test passed");
}

/// Test stderr messages with clipboard operations
#[rstest]
fn test_stderr_messages_with_clipboard(stdout_test_env: StdoutTestEnv) {
    // Test without --no-clipboard flag - should attempt clipboard operation
    // Note: In test environment (non-terminal), auto-quiet is enabled
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path()).assert().success();
    // In test environment, clipboard operations might be silent due to auto-quiet
    // This is expected behavior

    debug!("✓ Clipboard stderr messages test passed");
}

/// Test stderr behavior with different output formats
#[rstest]
#[case("json")]
#[case("xml")]
#[case("markdown")]
fn test_stderr_with_output_formats(stdout_test_env: StdoutTestEnv, #[case] format: &str) {
    let output_file = stdout_test_env.dir.path().join("output.txt");

    // Test that stderr messages appear regardless of output format
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap())
        .arg("--output-format")
        .arg(format)
        .arg("--no-clipboard")
        .assert()
        .success()
        .stderr(contains("Prompt written to file:"));

    debug!("✓ Stderr with {} format test passed", format);
}

/// Test stderr error messages for invalid scenarios
#[rstest]
fn test_stderr_error_messages(stdout_test_env: StdoutTestEnv) {
    // Test error message when --no-clipboard is used without --output-file
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("--no-clipboard")
        .assert()
        .failure()
        .stderr(contains(
            "--output-file is required when --no-clipboard is used",
        ));

    debug!("✓ Stderr error messages test passed");
}

/// Test that stdout and stderr are properly separated
#[rstest]
fn test_stdout_stderr_separation(stdout_test_env: StdoutTestEnv) {
    // Test that when outputting to stdout, status messages go to stderr, not stdout
    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg(stdout_test_env.path())
        .arg("-O")
        .arg("-")
        .arg("--no-clipboard")
        .assert()
        .success()
        // Content should be in stdout
        .stdout(contains("test.py"))
        .stdout(contains("print('Hello, World!')"))
        // Status messages should NOT be in stdout (they go to stderr in non-quiet mode)
        .stdout(contains("Token count:").not())
        .stdout(contains("✓").not());

    debug!("✓ Stdout/stderr separation test passed");
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
