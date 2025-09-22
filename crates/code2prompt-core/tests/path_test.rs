//! # Path Module Tests
//!
//! Tests for path traversal, directory structure handling, and file processing.
//! Uses rstest for parameterized testing and fixtures for test environment setup.

use code2prompt_core::{
    configuration::Code2PromptConfig,
    path::{EntryMetadata, traverse_directory},
};
use git2::Repository;
use rstest::*;
use serde_json::Value;
use std::{
    fs::{self},
    path::Path,
};
use tempfile::{TempDir, tempdir};

// ~~~ Fixtures ~~~

/// Creates a temporary directory with a git repository and test files
#[fixture]
fn git_repo_with_files() -> TempDir {
    let dir = tempdir().expect("Failed to create temp dir");
    let _repo = Repository::init(dir.path()).expect("Failed to init git repo");

    // Create test files, including one in target/
    let files = vec![
        ("src/main.rs", "// Main file"),
        ("target/debug/app", "// Binary in target/"),
        (".gitignore", "target/\n*.log"),
        ("README.md", "# Project Code2prompt"),
    ];

    for (path, content) in files {
        let full_path = dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create dir");
        }
        fs::write(full_path, content).expect("Failed to write file");
    }
    dir
}

/// Creates a simple directory structure without git
#[fixture]
fn simple_dir_structure() -> TempDir {
    let dir = tempdir().expect("Failed to create temp dir");

    let files = vec![
        ("file1.txt", "Content 1"),
        ("subdir/file2.txt", "Content 2"),
        ("subdir/nested/file3.txt", "Content 3"),
    ];

    for (path, content) in files {
        let full_path = dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create dir");
        }
        fs::write(full_path, content).expect("Failed to write file");
    }

    dir
}

/// Helper to create a basic config for testing
fn base_config(path: &Path) -> Code2PromptConfig {
    Code2PromptConfig::builder()
        .path(path.to_path_buf())
        .build()
        .expect("Failed to build config")
}

// ~~~ Test Helpers ~~~

/// Checks if a file exists in the output JSON
fn file_exists(files: &[Value], path: &str) -> bool {
    files.iter().any(|file| {
        file.get("path")
            .and_then(|p| p.as_str())
            .map(|p| p.contains(path))
            .unwrap_or(false)
    })
}

/// Gets metadata for a specific file
fn get_metadata(files: &[Value], path: &str) -> Option<EntryMetadata> {
    files
        .iter()
        .find(|file| {
            file.get("path")
                .and_then(|p| p.as_str())
                .map(|p| p.contains(path))
                .unwrap_or(false)
        })
        .and_then(|file| {
            file.get("metadata")
                .and_then(|m| serde_json::from_value(m.clone()).ok())
        })
}

// ~~~ Tests ~~~

#[cfg(test)]
mod tests {
    use super::*;

    // ~~~ Basic Traversal Tests ~~~

    #[rstest]
    fn test_basic_traversal(simple_dir_structure: TempDir) {
        let config = base_config(simple_dir_structure.path());
        let (tree_str, files) = traverse_directory(&config, None).unwrap();

        // Check tree contains all files
        assert!(tree_str.contains("file1.txt"));
        assert!(tree_str.contains("subdir"));
        assert!(tree_str.contains("file2.txt"));

        // Check files are processed
        assert_eq!(files.len(), 3);
        assert!(file_exists(&files, "file1.txt"));
        assert!(file_exists(&files, "file2.txt"));
        assert!(file_exists(&files, "file3.txt"));
    }

    // ~~~ Git Ignore Tests ~~~

    #[rstest]
    fn test_respects_gitignore(git_repo_with_files: TempDir) {
        let config = Code2PromptConfig::builder()
            .path(git_repo_with_files.path().to_path_buf())
            .no_ignore(false) // Respect .gitignore
            .build()
            .unwrap();

        let (_, files) = traverse_directory(&config, None).unwrap();

        // Verify target/ files are excluded
        assert!(!file_exists(&files, "target/debug/app"));

        // Verify non-ignored files are included
        assert!(file_exists(&files, "src/main.rs"));
        assert!(file_exists(&files, "README.md"));
    }

    #[rstest]
    fn test_ignores_gitignore_when_disabled(git_repo_with_files: TempDir) {
        let config = Code2PromptConfig::builder()
            .path(git_repo_with_files.path().to_path_buf())
            .no_ignore(true)
            .build()
            .unwrap();

        let (_, files) = traverse_directory(&config, None).unwrap();

        assert!(file_exists(&files, "src/main.rs"));
        assert!(file_exists(&files, "README.md"));
        assert!(file_exists(&files, "target/debug/app"));
    }

    // ~~~ Hidden Files Tests ~~~
    #[rstest]
    fn test_excludes_hidden_files_by_default(simple_dir_structure: TempDir) {
        // Add a hidden file
        fs::write(simple_dir_structure.path().join(".hidden"), "secret").unwrap();

        let config = base_config(simple_dir_structure.path());
        let (tree_str, files) = traverse_directory(&config, None).unwrap();

        // Hidden file should not appear
        assert!(!tree_str.contains(".hidden"));
        assert!(!file_exists(&files, ".hidden"));
    }

    #[rstest]
    fn test_includes_hidden_files_when_enabled(simple_dir_structure: TempDir) {
        // Add a hidden file
        fs::write(simple_dir_structure.path().join(".hidden"), "secret").unwrap();

        let config = Code2PromptConfig::builder()
            .path(simple_dir_structure.path().to_path_buf())
            .hidden(true)
            .build()
            .unwrap();

        let (tree_str, files) = traverse_directory(&config, None).unwrap();

        // Hidden file should appear
        assert!(tree_str.contains(".hidden"));
        assert!(file_exists(&files, ".hidden"));
    }

    // ~~~ File Content Tests ~~~
    #[rstest]
    fn test_file_content_processing(simple_dir_structure: TempDir) {
        let config = Code2PromptConfig::builder()
            .path(simple_dir_structure.path().to_path_buf())
            .line_numbers(true)
            .build()
            .unwrap();

        let (_, files) = traverse_directory(&config, None).unwrap();

        // Find file1.txt and check its content
        if let Some(file) = files.iter().find(|f| {
            f.get("path")
                .and_then(|p| p.as_str())
                .map(|p| p.contains("file1.txt"))
                .unwrap_or(false)
        }) {
            let code = file.get("code").and_then(|c| c.as_str()).unwrap();
            assert!(code.contains("Content 1"));
            assert!(code.contains("1 |")); // Line numbers should be present
        } else {
            panic!("file1.txt not found in output");
        }
    }

    // ~~~ Metadata Tests ~~~

    #[rstest]
    fn test_file_metadata(simple_dir_structure: TempDir) {
        let config = base_config(simple_dir_structure.path());
        let (_, files) = traverse_directory(&config, None).unwrap();

        // Check metadata for file1.txt
        if let Some(metadata) = get_metadata(&files, "file1.txt") {
            assert!(!metadata.is_dir);
            assert!(!metadata.is_symlink);
        } else {
            panic!("Metadata not found for file1.txt");
        }
    }

    // ~~~ Absolute vs Relative Path Tests ~~~

    #[rstest]
    fn test_relative_paths_by_default(simple_dir_structure: TempDir) {
        let config = base_config(simple_dir_structure.path());
        let (_, files) = traverse_directory(&config, None).unwrap();

        // Paths should be relative by default
        assert!(files.iter().all(|file| {
            file.get("path")
                .and_then(|p| p.as_str())
                .map(|p| !p.starts_with('/'))
                .unwrap_or(false)
        }));
    }

    #[rstest]
    fn test_absolute_paths_when_enabled(simple_dir_structure: TempDir) {
        let config = Code2PromptConfig::builder()
            .path(simple_dir_structure.path().to_path_buf())
            .absolute_path(true)
            .build()
            .unwrap();

        let (_, files) = traverse_directory(&config, None).unwrap();

        // Paths should be absolute when enabled
        let abs_path = simple_dir_structure.path().canonicalize().unwrap();
        assert!(files.iter().all(|file| {
            file.get("path")
                .and_then(|p| p.as_str())
                .map(|p| p.starts_with(abs_path.to_str().unwrap()))
                .unwrap_or(false)
        }));
    }

    // ~~~ Symlink Tests ~~~

    // #[rstest]
    // #[cfg(unix)] // Only run on Unix
    // fn test_symlink_following_disabled_by_default(simple_dir_structure: TempDir) {
    //     // Create a symlink to file1.txt
    //     let link_path = simple_dir_structure.path().join("link_to_file");
    //     std::os::unix::fs::symlink(simple_dir_structure.path().join("file1.txt"), &link_path)
    //         .unwrap();

    //     // Traverse with follow_symlinks=false (default)
    //     let config = base_config(simple_dir_structure.path());
    //     let (tree_str, files) = traverse_directory(&config, None).unwrap();

    //     // 1. Symlink should appear in the tree (it's a directory entry)
    //     assert!(tree_str.contains("link_to_file"));

    //     // 2. But its *content* (file1.txt's content) should NOT appear in `files`
    //     //    because we didn't follow the symlink.
    //     assert!(!file_exists(&files, "link_to_file"));

    //     // 3. file1.txt should still exist independently
    //     assert!(file_exists(&files, "file1.txt"));
    // }

    #[rstest]
    fn test_symlink_following_when_enabled(simple_dir_structure: TempDir) {
        let link_path = simple_dir_structure.path().join("link_to_file");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(simple_dir_structure.path().join("file1.txt"), &link_path)
                .unwrap();
        }

        let config = Code2PromptConfig::builder()
            .path(simple_dir_structure.path().to_path_buf())
            .follow_symlinks(true)
            .build()
            .unwrap();

        let (tree_str, _) = traverse_directory(&config, None).unwrap();

        // Symlink should be followed when enabled
        #[cfg(unix)]
        assert!(tree_str.contains("link_to_file"));
    }
}
