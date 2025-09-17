//! Integration tests for the session with simplified file selection

use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::session::Code2PromptSession;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test directory structure
        fs::create_dir_all(base_path.join("src")).unwrap();
        fs::create_dir_all(base_path.join("tests")).unwrap();

        // Create test files
        fs::write(base_path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(base_path.join("src/lib.rs"), "pub mod utils;").unwrap();
        fs::write(base_path.join("src/utils.rs"), "pub fn helper() {}").unwrap();
        fs::write(base_path.join("tests/test_main.rs"), "#[test] fn test() {}").unwrap();
        fs::write(base_path.join("README.md"), "# Test Project").unwrap();

        temp_dir
    }

    #[test]
    fn test_session_select_deselect_file() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["*".to_string()]) // Exclude everything initially
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");

        // Initially, no files should be selected (excluded by pattern)
        assert!(!session.is_file_selected(&main_rs_relative));
        assert!(session.get_selected_files().unwrap().is_empty());

        // Select the file using relative path (user action overrides pattern)
        session.select_file(main_rs_relative.clone());
        assert!(session.is_file_selected(&main_rs_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 1);

        // Deselect the file
        session.deselect_file(main_rs_relative.clone());
        assert!(!session.is_file_selected(&main_rs_relative));
        assert!(session.get_selected_files().unwrap().is_empty());
    }

    #[test]
    fn test_session_multiple_files() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");
        let utils_rs_relative = std::path::PathBuf::from("src/utils.rs");
        let readme_relative = std::path::PathBuf::from("README.md");

        // Select multiple files using relative paths
        session.select_file(main_rs_relative.clone());
        session.select_file(utils_rs_relative.clone());
        session.select_file(readme_relative.clone());

        assert!(session.is_file_selected(&main_rs_relative));
        assert!(session.is_file_selected(&utils_rs_relative));
        assert!(session.is_file_selected(&readme_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 3);

        // Deselect one file
        session.deselect_file(utils_rs_relative.clone());
        assert!(session.is_file_selected(&main_rs_relative));
        assert!(!session.is_file_selected(&utils_rs_relative));
        assert!(session.is_file_selected(&readme_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 2);
    }

    #[test]
    fn test_session_multiple_file_selection() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");
        let utils_rs_relative = std::path::PathBuf::from("src/utils.rs");

        // Select multiple files individually using relative paths
        session.select_file(main_rs_relative.clone());
        session.select_file(utils_rs_relative.clone());

        assert!(session.is_file_selected(&main_rs_relative));
        assert!(session.is_file_selected(&utils_rs_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 2);
    }

    #[test]
    fn test_session_clear_user_actions() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["*".to_string()]) // Exclude everything initially
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");
        let utils_rs_relative = std::path::PathBuf::from("src/utils.rs");

        // Select some files using relative paths (user actions override exclude patterns)
        session.select_file(main_rs_relative.clone());
        session.select_file(utils_rs_relative.clone());
        assert_eq!(session.get_selected_files().unwrap().len(), 2);

        // Clear all user actions (reset to pattern-only behavior)
        session.clear_user_actions();
        // After clearing user actions, files should be excluded by the exclude pattern
        assert!(session.get_selected_files().unwrap().is_empty());
    }

    #[test]
    fn test_session_add_patterns() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);

        // Initially no patterns
        assert!(session.config.include_patterns.is_empty());
        assert!(session.config.exclude_patterns.is_empty());

        // Add patterns
        session.add_include_pattern("*.rs".to_string());
        session.add_exclude_pattern("**/test*".to_string());

        assert_eq!(session.config.include_patterns.len(), 1);
        assert_eq!(session.config.exclude_patterns.len(), 1);
        assert_eq!(session.config.include_patterns[0], "*.rs");
        assert_eq!(session.config.exclude_patterns[0], "**/test*");
    }

    #[test]
    fn test_session_relative_path_handling() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_absolute = temp_dir.path().join("src/main.rs");
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");

        // Select using absolute path
        session.select_file(main_rs_absolute.clone());

        // Should be found using both absolute and relative paths
        assert!(session.is_file_selected(&main_rs_absolute));
        assert!(session.is_file_selected(&main_rs_relative));

        // The stored path should be relative
        let selected_files = session.get_selected_files().unwrap();
        assert_eq!(selected_files.len(), 1);
        assert_eq!(selected_files[0], main_rs_relative);
    }
}
