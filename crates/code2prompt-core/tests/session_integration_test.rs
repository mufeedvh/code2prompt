//! Integration tests for the session with layered selection

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
    fn test_session_toggle_file() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["*.rs".to_string()])
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_path = temp_dir.path().join("src/main.rs");

        // Initially, main.rs should be excluded by pattern
        assert!(!session.is_file_included(&main_rs_path));

        // Toggle to include it explicitly
        session.toggle_file(main_rs_path.clone());
        assert!(session.is_file_included(&main_rs_path));

        // Toggle again to exclude it explicitly
        session.toggle_file(main_rs_path.clone());
        assert!(!session.is_file_included(&main_rs_path));
    }

    #[test]
    fn test_session_explicit_include_exclude() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .include_patterns(vec!["src/**".to_string()])
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let utils_rs_path = temp_dir.path().join("src/utils.rs");

        // Initially, utils.rs should be included by pattern
        assert!(session.is_file_included(&utils_rs_path));

        // Explicitly exclude it
        session.exclude_file(utils_rs_path.clone());
        assert!(!session.is_file_included(&utils_rs_path));

        // Explicitly include it back
        session.include_file(utils_rs_path.clone());
        assert!(session.is_file_included(&utils_rs_path));
    }

    #[test]
    fn test_session_clear_explicit_overrides() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["*.rs".to_string()])
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_path = temp_dir.path().join("src/main.rs");

        // Explicitly include main.rs despite exclude pattern
        session.include_file(main_rs_path.clone());
        assert!(session.is_file_included(&main_rs_path));

        // Clear explicit overrides - should fall back to patterns
        session.clear_explicit_overrides();
        assert!(!session.is_file_included(&main_rs_path)); // Excluded by pattern again
    }

    #[test]
    fn test_session_add_patterns() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_path = temp_dir.path().join("src/main.rs");
        let readme_path = temp_dir.path().join("README.md");

        // Initially, everything should be included (no patterns)
        assert!(session.is_file_included(&main_rs_path));
        assert!(session.is_file_included(&readme_path));

        // Add exclude pattern for Rust files
        session.add_exclude_pattern("*.rs".to_string());
        assert!(!session.is_file_included(&main_rs_path));
        assert!(session.is_file_included(&readme_path));

        // Add include pattern - but exclude still takes precedence in pattern logic
        session.add_include_pattern("src/**".to_string());
        assert!(!session.is_file_included(&main_rs_path)); // Still excluded because exclude pattern takes precedence

        // README.md is not matched by include patterns, so when include patterns exist,
        // files not matching include patterns are excluded
        assert!(!session.is_file_included(&readme_path));
    }

    #[test]
    fn test_folder_propagation() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["src/**".to_string()])
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let src_path = temp_dir.path().join("src");
        let main_rs_path = temp_dir.path().join("src/main.rs");
        let utils_rs_path = temp_dir.path().join("src/utils.rs");

        // Initially, all src files should be excluded
        assert!(!session.is_file_included(&main_rs_path));
        assert!(!session.is_file_included(&utils_rs_path));

        // Explicitly include the src folder - should propagate to children
        session.include_file(src_path);
        assert!(session.is_file_included(&main_rs_path));
        assert!(session.is_file_included(&utils_rs_path));
    }
}
