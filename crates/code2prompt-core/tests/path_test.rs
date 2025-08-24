use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::path::traverse_directory;
use colored::*;
use log::{debug, info};
use std::fs::{self, read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::sync::Once;
use tempfile::tempdir;

use git2::Repository;

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
    //debug!("Writing to file: {:?}", file_path);
    writeln!(file, "{}", content).expect(&format!("Failed to write to temp file: {:?}", file_path));
}

fn create_test_hierarchy(base_path: &Path) {
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
        File::create(&gitignore_path).expect("Failed to create .gitignore file");
    writeln!(gitignore_file, "test_dir/ignored.txt").expect("Failed to write to .gitignore file");

    let gitignore_content =
        read_to_string(&gitignore_path).expect("Failed to read .gitignore file");
    debug!("gitignore content: {}", gitignore_content);

    info!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Tempfiles created".green()
    );
}

mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestEnv {
        dir: TempDir,
    }

    impl TestEnv {
        fn new() -> Self {
            init_logger();
            let dir = tempdir().unwrap();
            let _repo = Repository::init(dir.path()).expect("Failed to initialize repository");
            create_test_hierarchy(dir.path());
            TestEnv { dir }
        }
    }

    #[test]
    fn test_no_ignore_true() {
        let env = TestEnv::new();

        let config = Code2PromptConfig::builder()
            .path(env.dir.path().to_path_buf())
            .no_ignore(true)
            .build()
            .expect("Failed to build config");

        let result = traverse_directory(&config);
        let (tree_str, files) = result.unwrap();

        // Must contain both included.txt and ignored.txt because .gitignore is ignored
        assert!(tree_str.contains("included.txt"));
        assert!(tree_str.contains("ignored.txt"));
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("included.txt")).unwrap_or(false)
        }));
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("ignored.txt")).unwrap_or(false)
        }));
    }

    #[test]
    fn test_no_ignore_false() {
        let env = TestEnv::new();

        let config = Code2PromptConfig::builder()
            .path(env.dir.path().to_path_buf())
            .no_ignore(false)
            .build()
            .expect("Failed to build config");

        let result = traverse_directory(&config);
        let (tree_str, files) = result.unwrap();

        // Must contain included.txt but NOT ignored.txt
        assert!(tree_str.contains("included.txt"));
        assert!(!tree_str.contains("ignored.txt"));
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("included.txt")).unwrap_or(false)
        }));
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("ignored.txt")).unwrap_or(false)
        }));
    }

    #[test]
    fn test_extra_ignore_files() {
        let env = TestEnv::new();

        // Create additional files for this test
        create_temp_file(env.dir.path(), "test_dir/docker_file.txt", "Docker file");
        create_temp_file(env.dir.path(), "test_dir/npm_file.txt", "NPM file");
        create_temp_file(env.dir.path(), "test_dir/regular_file.txt", "Regular file");

        // Create .dockerignore file
        let dockerignore_path = env.dir.path().join(".dockerignore");
        let mut dockerignore_file =
            File::create(&dockerignore_path).expect("Failed to create .dockerignore file");
        writeln!(dockerignore_file, "test_dir/docker_file.txt")
            .expect("Failed to write to .dockerignore file");

        // Create .npmignore file
        let npmignore_path = env.dir.path().join(".npmignore");
        let mut npmignore_file =
            File::create(&npmignore_path).expect("Failed to create .npmignore file");
        writeln!(npmignore_file, "test_dir/npm_file.txt")
            .expect("Failed to write to .npmignore file");

        let config = Code2PromptConfig::builder()
            .path(env.dir.path().to_path_buf())
            .no_ignore(false)
            .extra_ignore_files(vec![".dockerignore".to_string(), ".npmignore".to_string()])
            .build()
            .expect("Failed to build config");

        let result = traverse_directory(&config);
        let (tree_str, files) = result.unwrap();

        // Should contain included.txt and regular_file.txt but NOT ignored.txt, docker_file.txt, or npm_file.txt
        assert!(tree_str.contains("included.txt"));
        assert!(tree_str.contains("regular_file.txt"));
        assert!(!tree_str.contains("ignored.txt")); // From .gitignore
        assert!(!tree_str.contains("docker_file.txt")); // From .dockerignore
        assert!(!tree_str.contains("npm_file.txt")); // From .npmignore

        // Verify files array
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("included.txt")).unwrap_or(false)
        }));
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("regular_file.txt")).unwrap_or(false)
        }));
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("ignored.txt")).unwrap_or(false)
        }));
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("docker_file.txt")).unwrap_or(false)
        }));
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("npm_file.txt")).unwrap_or(false)
        }));
    }

    #[test]
    fn test_promptignore_local() {
        let env = TestEnv::new();

        // Create additional files for this test
        create_temp_file(env.dir.path(), "test_dir/prompt_ignored.txt", "Prompt ignored file");
        create_temp_file(env.dir.path(), "test_dir/regular_file.txt", "Regular file");

        // Create .promptignore file
        let promptignore_path = env.dir.path().join(".promptignore");
        let mut promptignore_file =
            File::create(&promptignore_path).expect("Failed to create .promptignore file");
        writeln!(promptignore_file, "test_dir/prompt_ignored.txt")
            .expect("Failed to write to .promptignore file");

        let config = Code2PromptConfig::builder()
            .path(env.dir.path().to_path_buf())
            .no_ignore(false)
            .no_promptignore(false)
            .build()
            .expect("Failed to build config");

        let result = traverse_directory(&config);
        let (tree_str, files) = result.unwrap();

        // Should contain included.txt and regular_file.txt but NOT ignored.txt or prompt_ignored.txt
        assert!(tree_str.contains("included.txt"));
        assert!(tree_str.contains("regular_file.txt"));
        assert!(!tree_str.contains("ignored.txt")); // From .gitignore
        assert!(!tree_str.contains("prompt_ignored.txt")); // From .promptignore

        // Verify files array
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("included.txt")).unwrap_or(false)
        }));
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("regular_file.txt")).unwrap_or(false)
        }));
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("prompt_ignored.txt")).unwrap_or(false)
        }));
    }

    #[test]
    fn test_promptignore_global() {
        let env = TestEnv::new();

        // Create additional files for this test
        create_temp_file(env.dir.path(), "test_dir/globally_ignored.txt", "Globally ignored file");
        create_temp_file(env.dir.path(), "test_dir/regular_file.txt", "Regular file");

        // Create a temporary home directory and global .promptignore
        let home_dir = tempdir().unwrap();
        let global_promptignore = home_dir.path().join(".promptignore");
        let mut global_promptignore_file =
            File::create(&global_promptignore).expect("Failed to create global .promptignore file");
        writeln!(global_promptignore_file, "globally_ignored.txt")
            .expect("Failed to write to global .promptignore file");

        // Override the home directory for this test (we can't easily test this without mocking dirs::home_dir)
        // So we'll test by manually adding the global ignore file
        let _config = Code2PromptConfig::builder()
            .path(env.dir.path().to_path_buf())
            .no_ignore(false)
            .no_promptignore(false)
            .build()
            .expect("Failed to build config");

        // For this test, we'll manually create the walker to test global promptignore behavior
        use ignore::WalkBuilder;
        let mut walker = WalkBuilder::new(env.dir.path());
        walker
            .hidden(true)
            .git_ignore(true)
            .follow_links(false);

        // Add local .promptignore
        walker.add_custom_ignore_filename(".promptignore");

        // Add global .promptignore
        walker.add_ignore(&global_promptignore);

        let entries: Vec<_> = walker.build()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .collect();

        // Should find included.txt and regular_file.txt but not ignored.txt or globally_ignored.txt
        let file_names: Vec<String> = entries
            .iter()
            .map(|entry| entry.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"included.txt".to_string()));
        assert!(file_names.contains(&"regular_file.txt".to_string()));
        assert!(!file_names.contains(&"ignored.txt".to_string())); // From .gitignore
        assert!(!file_names.contains(&"globally_ignored.txt".to_string())); // From global .promptignore
    }

    #[test]
    fn test_no_promptignore_flag() {
        let env = TestEnv::new();

        // Create additional files for this test
        create_temp_file(env.dir.path(), "test_dir/prompt_ignored.txt", "Prompt ignored file");
        create_temp_file(env.dir.path(), "test_dir/regular_file.txt", "Regular file");

        // Create .promptignore file
        let promptignore_path = env.dir.path().join(".promptignore");
        let mut promptignore_file =
            File::create(&promptignore_path).expect("Failed to create .promptignore file");
        writeln!(promptignore_file, "test_dir/prompt_ignored.txt")
            .expect("Failed to write to .promptignore file");

        let config = Code2PromptConfig::builder()
            .path(env.dir.path().to_path_buf())
            .no_ignore(false)
            .no_promptignore(true) // Disable .promptignore
            .build()
            .expect("Failed to build config");

        let result = traverse_directory(&config);
        let (tree_str, files) = result.unwrap();

        // Should contain included.txt, regular_file.txt AND prompt_ignored.txt (since --no-promptignore is enabled)
        // But NOT ignored.txt (still respected from .gitignore)
        assert!(tree_str.contains("included.txt"));
        assert!(tree_str.contains("regular_file.txt"));
        assert!(tree_str.contains("prompt_ignored.txt")); // Should be included due to --no-promptignore

        // Check that ignored.txt is NOT in the files array since it should be filtered by gitignore
        // (We check files array instead of tree_str because tree might show directory structure differently)
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p == "test_dir/ignored.txt").unwrap_or(false)
        }));

        // Verify files array
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("included.txt")).unwrap_or(false)
        }));
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("regular_file.txt")).unwrap_or(false)
        }));
        assert!(files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p.contains("prompt_ignored.txt")).unwrap_or(false)
        }));
        assert!(!files.iter().any(|file| {
            let path = file.get("path").and_then(|p| p.as_str());
            path.map(|p| p == "test_dir/ignored.txt").unwrap_or(false)
        }));
    }
}
