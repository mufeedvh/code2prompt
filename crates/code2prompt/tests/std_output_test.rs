use assert_cmd::Command;
use colored::*;
use log::{debug, info};
use predicates::prelude::*;
use predicates::str::contains;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Once;
use tempfile::tempdir;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init()
            .expect("Failed to initialize logger");
    });
}

fn create_temp_file(dir: &Path, name: &str, content: &str) {
    let file_path = dir.join(name);
    let parent_dir = file_path.parent().unwrap();
    fs::create_dir_all(parent_dir).expect(&format!("Failed to create directory: {:?}", parent_dir));
    let mut file =
        File::create(&file_path).expect(&format!("Failed to create temp file: {:?}", file_path));
    writeln!(file, "{}", content).expect(&format!("Failed to write to temp file: {:?}", file_path));
}

fn create_simple_test_files(base_path: &Path) {
    let files = vec![
        ("test.py", "print('Hello, World!')"),
        ("README.md", "# Test Project\nThis is a test."),
        ("config.json", r#"{"name": "test", "version": "1.0.0"}"#),
    ];

    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }
    info!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Test files created".green()
    );
}

mod stdout_tests {
    use super::*;
    use tempfile::TempDir;

    struct TestEnv {
        dir: TempDir,
    }

    impl TestEnv {
        fn new() -> Self {
            init_logger();
            let dir = tempdir().unwrap();
            create_simple_test_files(dir.path());
            TestEnv { dir }
        }

        fn path(&self) -> &str {
            self.dir.path().to_str().unwrap()
        }
    }

    #[test]
    fn test_stdout_output_default() {
        let env = TestEnv::new();

        // Test: code2prompt path (without --no-clipboard, outputs status to stdout, prompt to clipboard)
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .assert()
            .success()
            .stdout(contains("Token count:"))
            .stdout(contains("Copied to clipboard successfully"));

        debug!("✓ Default stdout output test passed");
    }

    #[test]
    fn test_stdout_output_explicit_dash() {
        let env = TestEnv::new();

        // Test: code2prompt path -O - (should output to stdout)
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .arg("-O")
            .arg("-")
            .arg("--no-clipboard")
            .assert()
            .success()
            .stdout(contains("test.py"))
            .stdout(contains("print('Hello, World!')"))
            .stdout(contains("README.md"))
            .stdout(contains("# Test Project"));

        debug!("✓ Explicit stdout (-O -) output test passed");
    }

    #[test]
    fn test_stdout_output_long_form() {
        let env = TestEnv::new();

        // Test: code2prompt path --output-file - (should output to stdout)
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .arg("--output-file")
            .arg("-")
            .arg("--no-clipboard")
            .assert()
            .success()
            .stdout(contains("test.py"))
            .stdout(contains("print('Hello, World!')"))
            .stdout(contains("README.md"))
            .stdout(contains("# Test Project"));

        debug!("✓ Long form stdout (--output-file -) output test passed");
    }

    #[test]
    fn test_conflicting_output_options_should_fail() {
        let env = TestEnv::new();

        // Test: Using both default stdout and explicit -O - should fail
        // This is a logical conflict - you can't output to stdout in two different ways
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .arg("-")
            .arg("-O")
            .arg("-")
            .arg("--no-clipboard")
            .assert()
            .failure();

        debug!("✓ Conflicting output options test passed (correctly failed)");
    }

    #[test]
    fn test_output_file_vs_stdout_conflict() {
        let env = TestEnv::new();
        let output_file = env.dir.path().join("output.txt");

        // Test: Using both output file and stdout should fail
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
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

    #[test]
    fn test_no_clipboard_requires_output_file() {
        let env = TestEnv::new();

        // Test: When --no-clipboard is used, --output-file is required
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .arg("--no-clipboard")
            .assert()
            .failure()
            .stderr(contains(
                "--output-file is required when --no-clipboard is used",
            ));

        debug!("✓ No clipboard requires output file test passed");
    }

    #[test]
    fn test_quiet_mode_with_stdout() {
        let env = TestEnv::new();

        // Test: Quiet mode should still output the prompt to stdout but suppress other messages
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .arg("--quiet")
            .arg("-O")
            .arg("-")
            .arg("--no-clipboard")
            .assert()
            .success()
            .stdout(contains("test.py"))
            .stdout(contains("print('Hello, World!')"))
            // Should not contain success messages in quiet mode
            .stdout(contains("✓").not());

        debug!("✓ Quiet mode with stdout test passed");
    }

    #[test]
    fn test_stdout_with_different_formats() {
        let env = TestEnv::new();

        // Test: Stdout should work with different output formats
        let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
        cmd.arg(env.path())
            .arg("--output-format")
            .arg("json")
            .arg("-O")
            .arg("-")
            .arg("--no-clipboard")
            .assert()
            .success()
            .stdout(contains("{"))
            .stdout(contains("\"files\""))
            .stdout(contains("test.py"));

        debug!("✓ Stdout with JSON format test passed");
    }
}
