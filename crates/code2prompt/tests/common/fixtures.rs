//! rstest fixtures for code2prompt integration tests

use super::test_env::*;
use colored::*;
use log::info;
use rstest::*;
use std::fs;

/// Fixture for basic test environment with standard file hierarchy
#[fixture]
pub fn basic_test_env() -> BasicTestEnv {
    let env = BasicTestEnv::new();
    create_standard_hierarchy(env.dir.path());
    env
}

/// Fixture for git test environment with gitignore setup
#[fixture]
pub fn git_test_env() -> GitTestEnv {
    let env = GitTestEnv::new();
    create_git_hierarchy(env.dir.path());
    env
}

/// Fixture for stdout test environment with simple files
#[fixture]
pub fn stdout_test_env() -> StdoutTestEnv {
    let env = StdoutTestEnv::new();
    create_simple_test_files(env.dir.path());
    env
}

/// Fixture for template test environment with code structure
#[fixture]
pub fn template_test_env() -> TemplateTestEnv {
    let env = TemplateTestEnv::new();
    create_test_codebase(env.dir.path());
    env
}

/// Create standard test hierarchy (lowercase/uppercase directories with various files)
pub fn create_standard_hierarchy(base_path: &std::path::Path) {
    let lowercase_dir = base_path.join("lowercase");
    let uppercase_dir = base_path.join("uppercase");

    fs::create_dir_all(&lowercase_dir).unwrap();
    fs::create_dir_all(&uppercase_dir).unwrap();

    let files = vec![
        ("lowercase/foo.py", "content foo.py"),
        ("lowercase/bar.py", "content bar.py"),
        ("lowercase/baz.py", "content baz.py"),
        ("lowercase/qux.txt", "content qux.txt"),
        ("lowercase/corge.txt", "content corge.txt"),
        ("lowercase/grault.txt", "content grault.txt"),
        ("uppercase/FOO.py", "CONTENT FOO.PY"),
        ("uppercase/BAR.py", "CONTENT BAR.PY"),
        ("uppercase/BAZ.py", "CONTENT BAZ.PY"),
        ("uppercase/QUX.txt", "CONTENT QUX.TXT"),
        ("uppercase/CORGE.txt", "CONTENT CORGE.TXT"),
        ("uppercase/GRAULT.txt", "CONTENT GRAULT.TXT"),
    ];

    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }

    info!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Standard test hierarchy created".green()
    );
}

/// Create git test hierarchy with gitignore
pub fn create_git_hierarchy(base_path: &std::path::Path) {
    let test_dir = base_path.join("test_dir");
    fs::create_dir_all(&test_dir).unwrap();

    let files = vec![
        ("test_dir/included.txt", "Included file"),
        ("test_dir/ignored.txt", "Ignored file"),
    ];

    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }

    // Create a .gitignore file
    let gitignore_path = base_path.join(".gitignore");
    let mut gitignore_file =
        std::fs::File::create(&gitignore_path).expect("Failed to create .gitignore file");
    use std::io::Write;
    writeln!(gitignore_file, "test_dir/ignored.txt").expect("Failed to write to .gitignore file");

    info!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Git test hierarchy created".green()
    );
}

/// Create simple test files for stdout tests
pub fn create_simple_test_files(base_path: &std::path::Path) {
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
        "Simple test files created".green()
    );
}

/// Create test codebase for template tests
pub fn create_test_codebase(base_path: &std::path::Path) {
    let files = vec![
        (
            "src/main.rs",
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        ),
        (
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
        ),
        (
            "tests/test.rs",
            "#[test]\nfn test_add() {\n    assert_eq!(3, add(1, 2));\n}",
        ),
    ];

    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }

    info!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Test codebase created".green()
    );
}
