//! Template integration tests for code2prompt
//!
//! This module tests template functionality, output formats,
//! and template rendering using rstest fixtures.

mod common;

use common::fixtures::*;
use common::*;
use log::debug;
use predicates::prelude::*;
use predicates::str::{contains, ends_with, starts_with};
use rstest::*;

/// Test different output format templates
#[rstest]
#[case("markdown", vec!["Source Tree:", "```rs", "fn main()", "Hello, world!"])]
#[case("xml", vec!["<directory>", "</file>", ".rs\"", "fn main()", "Hello, world!"])]
fn test_output_format_templates(
    template_test_env: TemplateTestEnv,
    #[case] format: &str,
    #[case] expected_content: Vec<&str>,
) {
    let mut cmd = template_test_env.command();
    cmd.arg(format!("--output-format={}", format))
        .assert()
        .success();

    let output = template_test_env.read_output();
    debug!("{} template output:\n{}", format, output);

    // Check format-specific content
    for expected in expected_content {
        assert!(
            contains(expected).eval(&output),
            "Expected '{}' in {} format output",
            expected,
            format
        );
    }
}

/// Test JSON output format (special case with structured output)
#[rstest]
fn test_json_output_format(template_test_env: TemplateTestEnv) {
    let mut cmd = template_test_env.command();
    cmd.arg("--output-format=json").assert().success();

    let output = template_test_env.read_output();
    debug!("JSON output format:\n{}", output);

    // JSON output should be structured
    assert!(starts_with("{").eval(&output));
    assert!(contains("\"directory_name\":").eval(&output));
    assert!(contains("\"prompt\": \"<directory>").eval(&output));
    assert!(ends_with("}").eval(&output));
}

/// Test that template fixture creates proper codebase structure
#[rstest]
fn test_template_fixture_setup(template_test_env: TemplateTestEnv) {
    // Verify that the fixture created the expected code structure
    let expected_files = vec![
        ("src/main.rs", "fn main()"),
        ("src/lib.rs", "pub fn add"),
        ("tests/test.rs", "#[test]"),
    ];

    for (file_path, expected_content) in expected_files {
        let file_path = template_test_env.dir.path().join(file_path);
        assert!(
            file_path.exists(),
            "Test file {} should exist",
            file_path.display()
        );

        let content =
            std::fs::read_to_string(&file_path).expect("Should be able to read test file");
        assert!(
            content.contains(expected_content),
            "File {} should contain '{}'",
            file_path.display(),
            expected_content
        );
    }

    debug!("✓ Template fixture setup test passed");
}

/// Test basic template rendering with default format
#[rstest]
fn test_basic_template_rendering(template_test_env: TemplateTestEnv) {
    let mut cmd = template_test_env.command();
    cmd.assert().success();

    let output = template_test_env.read_output();
    debug!("Basic template rendering output:\n{}", output);

    // Should contain code from all test files
    assert!(contains("fn main()").eval(&output));
    assert!(contains("Hello, world!").eval(&output));
    assert!(contains("pub fn add").eval(&output));
    assert!(contains("#[test]").eval(&output));
    assert!(contains("assert_eq!").eval(&output));
}

/// Test template with different file extensions
#[rstest]
fn test_template_with_file_extensions(template_test_env: TemplateTestEnv) {
    let mut cmd = template_test_env.command();
    cmd.assert().success();

    let output = template_test_env.read_output();
    debug!("Template with file extensions output:\n{}", output);

    // Should properly identify and format Rust files
    assert!(contains("src/main.rs").eval(&output));
    assert!(contains("src/lib.rs").eval(&output));
    assert!(contains("tests/test.rs").eval(&output));
}

/// Test template output contains proper structure
#[rstest]
fn test_template_output_structure(template_test_env: TemplateTestEnv) {
    let mut cmd = template_test_env.command();
    cmd.assert().success();

    let output = template_test_env.read_output();
    debug!("Template output structure:\n{}", output);

    // Should contain directory structure information
    assert!(contains("src").eval(&output));
    assert!(contains("tests").eval(&output));

    // Should contain file content
    assert!(!output.trim().is_empty(), "Output should not be empty");

    // Should be properly formatted (not just raw concatenation)
    let line_count = output.lines().count();
    assert!(
        line_count > 10,
        "Output should have substantial content with multiple lines"
    );
}

/// Test template with include/exclude filters
#[rstest]
#[case("--include=*.rs", vec!["src/main.rs", "src/lib.rs", "tests/test.rs"])]
#[case("--exclude=**/test.rs", vec!["src/main.rs", "src/lib.rs"])]
#[case("--include=src/**", vec!["src/main.rs", "src/lib.rs"])]
fn test_template_with_filters(
    template_test_env: TemplateTestEnv,
    #[case] filter_arg: &str,
    #[case] expected_files: Vec<&str>,
) {
    let mut cmd = template_test_env.command();
    cmd.arg(filter_arg).assert().success();

    let output = template_test_env.read_output();
    debug!("Template with filter '{}' output:\n{}", filter_arg, output);

    // Should contain expected files
    for expected_file in expected_files {
        assert!(
            contains(expected_file).eval(&output),
            "Expected file '{}' with filter '{}'",
            expected_file,
            filter_arg
        );
    }
}

/// Test template command creation
#[rstest]
fn test_template_command_creation(template_test_env: TemplateTestEnv) {
    // Test that our fixture creates working commands
    let mut cmd = template_test_env.command();
    cmd.assert().success();

    // Verify output file was created and is readable
    let output = template_test_env.read_output();
    assert!(!output.is_empty(), "Template output should not be empty");

    // Verify the output file exists
    assert!(
        template_test_env.output_file_exists(),
        "Output file should exist after command execution"
    );
}
