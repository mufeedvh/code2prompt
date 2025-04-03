use assert_cmd::Command;
use colored::*;
use log::{debug, info};
use predicates::prelude::*;
use predicates::str::contains;
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

fn read_output_file(dir: &Path, file_name: &str) -> String {
    let file_path = dir.join(file_name);
    read_to_string(&file_path).expect(&format!("Failed to read output file: {:?}", file_path))
}

mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestEnv {
        dir: TempDir,
        output_file: String,
    }

    impl TestEnv {
        fn new() -> Self {
            init_logger();
            let dir = tempdir().unwrap();
            let _repo = Repository::init(dir.path()).expect("Failed to initialize repository");

            create_test_hierarchy(dir.path());
            let output_file = dir.path().join("output.txt").to_str().unwrap().to_string();
            TestEnv { dir, output_file }
        }

        fn command(&self) -> Command {
            let mut cmd =
                Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
            cmd.arg(&self.dir.path().to_str().unwrap())
                .arg("--output-file")
                .arg(&self.output_file)
                .arg("--no-clipboard");
            cmd
        }

        fn read_output(&self) -> String {
            read_output_file(self.dir.path(), "output.txt")
        }
    }

    #[test]
    fn test_gitignore() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.assert().success();

        let output = env.read_output();
        debug!("Test --no-ignore flag output:\n{}", output);
        assert!(contains("included.txt").eval(&output));
        assert!(contains("Included file").eval(&output));
        assert!(contains("ignored.txt").not().eval(&output));
        assert!(contains("Ignored file").not().eval(&output));
    }

    #[test]
    fn test_gitignore_no_ignore() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--no-ignore").assert().success();

        let output2 = env.read_output();
        debug!("Test --no-ignore flag output:\n{}", output2);
        assert!(contains("included.txt").eval(&output2));
        assert!(contains("Included file").eval(&output2));
        assert!(contains("ignored.txt").eval(&output2));
        assert!(contains("Ignored file").eval(&output2));
    }
}
