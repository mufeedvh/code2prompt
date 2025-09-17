use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::path::traverse_directory;
use colored::*;
use log::{debug, info};
use std::fs::{self, File, read_to_string};
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

        let result = traverse_directory(&config, None);
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

        let result = traverse_directory(&config, None);
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
}
