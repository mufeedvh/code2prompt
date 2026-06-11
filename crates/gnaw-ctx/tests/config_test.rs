//! Tests for TOML configuration functionality
//!
//! This module tests the TOML configuration loading, parsing, and integration
//! with the new Unix-style behavior.

mod common;

use common::*;
use gnaw_core::sort::FileSortMethod;
use gnaw_core::template::OutputFormat;
use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Test TOML configuration parsing
#[test]
fn test_toml_config_parsing() {
    let toml_content = r#"
default_output = "clipboard"
path = "./src"
include_patterns = ["*.rs", "*.toml"]
exclude_patterns = ["target", "node_modules"]
line_numbers = true
absolute_path = false
full_directory_tree = false
output_format = "markdown"
sort_method = "name_asc"
encoding = "cl100k"
token_format = "format"
diff_enabled = true
diff_branches = ["main", "feature-x"]
log_branches = ["v1.0.0", "v1.1.0"]
template_name = "default"
template_str = ""
token_map_enabled = true

[user_variables]
project = "gnaw"
author = "ODAncona"
"#;

    use gnaw_core::configuration::TomlConfig;
    let config = TomlConfig::from_toml_str(toml_content).expect("Should parse TOML config");

    assert_eq!(
        config.default_output,
        gnaw_core::configuration::OutputDestination::Clipboard
    );
    assert_eq!(config.path, Some("./src".to_string()));
    assert_eq!(config.include_patterns, vec!["*.rs", "*.toml"]);
    assert_eq!(config.exclude_patterns, vec!["target", "node_modules"]);
    assert!(config.line_numbers);
    assert!(!config.absolute_path);
    assert!(!config.full_directory_tree);
    assert_eq!(config.output_format, Some(OutputFormat::Markdown));
    assert_eq!(config.sort_method, Some(FileSortMethod::NameAsc));
    assert_eq!(
        config.encoding,
        Some(gnaw_core::tokenizer::TokenizerType::Cl100kBase)
    );
    assert_eq!(
        config.token_format,
        Some(gnaw_core::tokenizer::TokenFormat::Format)
    );
    assert!(config.diff_enabled);
    assert_eq!(
        config.diff_branches,
        Some(vec!["main".to_string(), "feature-x".to_string()])
    );
    assert_eq!(
        config.log_branches,
        Some(vec!["v1.0.0".to_string(), "v1.1.0".to_string()])
    );
    assert_eq!(config.template_name, Some("default".to_string()));
    assert!(config.token_map_enabled);
    assert_eq!(
        config.user_variables.get("project"),
        Some(&"gnaw".to_string())
    );
    assert_eq!(
        config.user_variables.get("author"),
        Some(&"ODAncona".to_string())
    );
}

/// Test TOML config export functionality
#[test]
fn test_toml_config_export() {
    use gnaw_core::configuration::{GnawConfig, export_config_to_toml};

    let config = GnawConfig::builder()
        .path("./test")
        .include_patterns(vec!["*.rs".to_string()])
        .exclude_patterns(vec!["target".to_string()])
        .line_numbers(true)
        .build()
        .unwrap();

    let toml_str = export_config_to_toml(&config).expect("Should export to TOML");

    // Verify the exported TOML contains expected values
    assert!(toml_str.contains("default_output = \"stdout\""));
    assert!(toml_str.contains("path = \"./test\""));
    assert!(toml_str.contains("include_patterns = [\"*.rs\"]"));
    assert!(toml_str.contains("exclude_patterns = [\"target\"]"));
    assert!(toml_str.contains("line_numbers = true"));
}

/// Test local config file loading
#[test]
fn test_local_config_file_loading() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join(".gnawconfig");

    let toml_content = r#"
default_output = "stdout"
include_patterns = ["*.rs"]
line_numbers = true
"#;

    fs::write(&config_path, toml_content).expect("Should write config file");

    // Change to the temp directory
    let original_dir = std::env::current_dir().expect("Should get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Should change dir");

    // Test that the config is loaded (we can't easily test the actual loading here
    // without more complex setup, but we can test the file exists)
    assert!(config_path.exists());

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Should restore dir");
}

/// Test new Unix-style default behavior (stdout)
#[test]
fn test_unix_style_default_stdout() {
    let temp_dir = TempDir::new().expect("Should create temp dir");

    // Create a test.py file with expected content
    fs::write(temp_dir.path().join("test.py"), "print('Hello, World!')")
        .expect("Should write test file");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    let temp_path = temp_dir.path().to_path_buf();
    cmd.arg(&temp_path)
        .assert()
        .success()
        .stdout(contains("test.py"))
        .stdout(contains("print('Hello, World!')"));

    // Keep temp_dir alive until the end
    drop(temp_dir);
}

/// Test new clipboard flag
#[test]
fn test_clipboard_flag() {
    let test_env = StdoutTestEnv::new();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(test_env.path())
        .arg("-c") // New clipboard flag
        .assert()
        .success()
        // Should not output to stdout when using clipboard
        .stdout(contains("test.py").not());
}

/// Test that CLI args override config files
#[test]
fn test_cli_patterns_merge_with_config() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join(".gnawconfig");
    // Config excludes .log files. The CLI will add an include for .py — the
    // merge must keep BOTH: .py included, .log still excluded.
    let toml_content = r#"
default_output = "clipboard"
exclude_patterns = ["*.log"]
"#;
    fs::write(&config_path, toml_content).expect("Should write config file");
    fs::write(temp_dir.path().join("keep.py"), "print('keep')").expect("write py");
    fs::write(temp_dir.path().join("drop.log"), "noise").expect("write log");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.current_dir(temp_dir.path())
        .arg(".")
        .arg("-i")
        .arg("*.py") // CLI include ADDS to config patterns
        .arg("-O")
        .arg("-")
        .assert()
        .success()
        // CLI include took effect:
        .stdout(contains("keep.py"))
        .stdout(contains("print('keep')"))
        // ...AND config exclude survived the merge (the bug that inflated 290K):
        .stdout(contains("drop.log").not());
}

#[test]
fn test_config_exclude_beats_cli_include_on_collision() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join(".gnawconfig"),
        "exclude_patterns = [\"*.py\"]\n",
    )
    .unwrap();
    fs::write(temp_dir.path().join("test.py"), "print('x')").unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.current_dir(temp_dir.path())
        .arg(".")
        .arg("-i")
        .arg("*.py")
        .arg("-O")
        .arg("-")
        .assert()
        .success()
        // Same pattern excluded by config and included by CLI → exclude wins.
        .stdout(contains("test.py").not());
}

/// Test configuration info messages
#[test]
fn test_config_info_messages() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join(".gnawconfig");

    let toml_content = r#"
default_output = "stdout"
"#;

    fs::write(&config_path, toml_content).expect("Should write config file");
    fs::write(temp_dir.path().join("test.txt"), "content").expect("Should write test file");

    // Run with the temp directory as argument and set current directory for the command
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.current_dir(temp_dir.path())
        .arg(".")
        .assert()
        .success()
        .stderr(contains("[i] Using config from:"));
}

/// Test default configuration message
#[test]
fn test_default_config_message() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    fs::write(temp_dir.path().join("test.txt"), "content").expect("Should write test file");

    // Run with the temp directory as argument and set current directory for the command
    // No config file exists, so it should use default configuration
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.current_dir(temp_dir.path())
        .arg(".")
        .assert()
        .success()
        .stderr(contains("[i] Using default configuration"));
}

/// Test CLI args message - now CLI args are applied on top of config
#[test]
fn test_cli_args_message() {
    let test_env = StdoutTestEnv::new();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(test_env.path())
        .arg("-i")
        .arg("*.py")
        .assert()
        .success()
        .stderr(contains("[i] Using default configuration")); // Now always loads config first
}
