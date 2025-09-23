//! Integration tests for code2prompt file filtering functionality
//!
//! This module tests the include/exclude patterns, file filtering,
//! and directory tree generation features using rstest fixtures.

mod common;

use common::*;
use log::debug;
use predicates::prelude::*;
use predicates::str::contains;
use rstest::*;

/// Test file filtering with various include/exclude patterns
#[rstest]
fn test_file_filtering(
    basic_test_env: BasicTestEnv,
    #[values(
        ("include_extensions", vec!["--include=*.py"], vec!["foo.py", "content foo.py", "FOO.py", "CONTENT FOO.PY"], vec!["content qux.txt"]),
        ("exclude_extensions", vec!["--exclude=*.txt"], vec!["foo.py", "content foo.py", "FOO.py", "CONTENT FOO.PY"], vec!["lowercase/qux.txt", "content qux.txt"]),
        ("include_files", vec!["--include=**/foo.py,**/bar.py"], vec!["foo.py", "content foo.py", "bar.py", "content bar.py"], vec!["lowercase/baz.py", "content baz.py"]),
        ("include_folders", vec!["--include=**/lowercase/**"], vec!["foo.py", "content foo.py", "baz.py", "content baz.py"], vec!["uppercase/FOO"]),
        ("exclude_files", vec!["--exclude=**/foo.py,**/bar.py"], vec!["baz.py", "content baz.py"], vec!["lowercase/foo.py", "content foo.py", "lowercase/bar.py", "content bar.py"]),
        ("exclude_folders", vec!["--exclude=**/uppercase/**"], vec!["foo.py", "content foo.py", "baz.py", "content baz.py"], vec!["CONTENT FOO.py"])
    )]
    test_case: (&str, Vec<&str>, Vec<&str>, Vec<&str>),
) {
    let (name, args, should_include, should_exclude) = test_case;

    let mut cmd = basic_test_env.command();
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert().success();

    let output = basic_test_env.read_output();
    debug!("Test {} output:\n{}", name, output);

    // Check that expected content is included
    for expected in should_include {
        assert!(
            contains(expected).eval(&output),
            "Test {}: Expected '{}' to be included in output",
            name,
            expected
        );
    }

    // Check that expected content is excluded
    for expected in should_exclude {
        assert!(
            contains(expected).not().eval(&output),
            "Test {}: Expected '{}' to be excluded from output",
            name,
            expected
        );
    }
}

/// Test include/exclude combination with exclude priority
#[rstest]
fn test_include_exclude_with_exclude_priority(basic_test_env: BasicTestEnv) {
    let mut cmd = basic_test_env.command();
    cmd.arg("--include=*.py,**/lowercase/**")
        .arg("--exclude=**/foo.py,**/uppercase/**")
        .assert()
        .success();

    let output = basic_test_env.read_output();
    debug!("Test include and exclude combinations output:\n{}", output);

    // Should include
    assert!(contains("lowercase/baz.py").eval(&output));
    assert!(contains("content baz.py").eval(&output));

    // Should exclude (exclude takes priority)
    assert!(contains("lowercase/foo.py").not().eval(&output));
    assert!(contains("content foo.py").not().eval(&output));
    assert!(contains("uppercase/FOO.py").not().eval(&output));
    assert!(contains("CONTENT FOO.PY").not().eval(&output));
}

/// Test with no filters (should include everything)
#[rstest]
fn test_no_filters(basic_test_env: BasicTestEnv) {
    let mut cmd = basic_test_env.command();
    cmd.assert().success();

    let output = basic_test_env.read_output();
    debug!("Test no filters output:\n{}", output);

    // Should include all files
    let expected_files = vec![
        "foo.py",
        "content foo.py",
        "baz.py",
        "content baz.py",
        "FOO.py",
        "CONTENT FOO.PY",
        "BAZ.py",
        "CONTENT BAZ.PY",
    ];

    for expected in expected_files {
        assert!(
            contains(expected).eval(&output),
            "Expected '{}' to be included when no filters are applied",
            expected
        );
    }
}

/// Test full directory tree generation
#[rstest]
fn test_full_directory_tree(basic_test_env: BasicTestEnv) {
    let mut cmd = basic_test_env.command();
    cmd.arg("--full-directory-tree")
        .arg("--exclude")
        .arg("**/uppercase/**")
        .assert()
        .success();

    let output = basic_test_env.read_output();
    debug!("Test full directory tree output:\n{}", output);

    // Should show directory structure
    assert!(contains("├── lowercase").eval(&output));
    assert!(contains("└── uppercase").eval(&output));

    // Should show files in tree format
    assert!(contains("├── foo.py").eval(&output));
    assert!(contains("├── bar.py").eval(&output));
    assert!(contains("├── baz.py").eval(&output));

    // Should show excluded directory structure but not content
    assert!(contains("├── FOO.py").eval(&output));
    assert!(contains("├── BAR.py").eval(&output));
    assert!(contains("├── BAZ.py").eval(&output));
    assert!(!contains("CONTENT BAR.PY").eval(&output));
}

/// Test brace expansion patterns
#[rstest]
fn test_brace_expansion(basic_test_env: BasicTestEnv) {
    let mut cmd = basic_test_env.command();
    cmd.arg("--include")
        .arg("lowercase/{foo.py,bar.py,baz.py}")
        .arg("--exclude")
        .arg("lowercase/{qux.txt,corge.txt,grault.txt}")
        .assert()
        .success();

    let output = basic_test_env.read_output();
    debug!("Test brace expansion output:\n{}", output);

    // Should include specified Python files
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.py").eval(&output));
    assert!(contains("content bar.py").eval(&output));
    assert!(contains("baz.py").eval(&output));
    assert!(contains("content baz.py").eval(&output));

    // Should exclude specified text files
    assert!(contains("qux.txt").not().eval(&output));
    assert!(contains("corge.txt").not().eval(&output));
    assert!(contains("grault.txt").not().eval(&output));
}

/// Test command creation helper
#[rstest]
fn test_command_helper(basic_test_env: BasicTestEnv) {
    // Test that our fixture creates working commands
    let mut cmd = basic_test_env.command();
    cmd.assert().success();

    // Verify output file was created and is readable
    let output = basic_test_env.read_output();
    assert!(!output.is_empty(), "Output should not be empty");
}
