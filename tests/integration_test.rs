use std::fs::{self, File, read_to_string};
use std::io::Write;
use std::path::Path;
use tempfile::{tempdir};
use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use colored::*;
use log::{info, debug};
use std::sync::Once;

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
    let mut file = File::create(&file_path).expect(&format!("Failed to create temp file: {:?}", file_path));
    debug!("Writing to file: {:?}", file_path);
    writeln!(file, "{}", content).expect(&format!("Failed to write to temp file: {:?}", file_path));
}

fn create_test_hierarchy(base_path: &Path) {
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
            create_test_hierarchy(dir.path());
            let output_file = dir.path().join("output.txt").to_str().unwrap().to_string();
            TestEnv { dir, output_file }
        }

        fn command(&self) -> Command {
            let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
            cmd.arg(&self.dir.path().to_str().unwrap())
               .arg("--output").arg(&self.output_file)
               .arg("--no-clipboard");
            cmd
        }

        fn read_output(&self) -> String {
            read_output_file(self.dir.path(), "output.txt")
        }
    }

    #[test]
    fn test_include_extensions() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--include=*.py").assert().success();

        let output = env.read_output();
        debug!("Test include extensions output:\n{}", output);
        assert!(contains("foo.py").eval(&output));
        assert!(contains("content foo.py").eval(&output));
        assert!(contains("FOO.py").eval(&output));
        assert!(contains("CONTENT FOO.PY").eval(&output));
        assert!(contains("content qux.txt").not().eval(&output));
    }

    #[test]
    fn test_exclude_extensions() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--exclude=*.txt").assert().success();

        let output = env.read_output();
        debug!("Test exclude files output:\n{}", output);
        assert!(contains("foo.py").eval(&output));
        assert!(contains("content foo.py").eval(&output));
        assert!(contains("FOO.py").eval(&output));
        assert!(contains("CONTENT FOO.PY").eval(&output));
        assert!(contains("lowercase/qux.txt").not().eval(&output));
        assert!(contains("content qux.txt").not().eval(&output));
    }

    #[test]
    fn test_include_files() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--include=**/foo.py,**/bar.py").assert().success();

        let output = env.read_output();
        debug!("Test include files output:\n{}", output);
        assert!(contains("foo.py").eval(&output));
        assert!(contains("content foo.py").eval(&output));
        assert!(contains("bar.py").eval(&output));
        assert!(contains("content bar.py").eval(&output));
        assert!(contains("lowercase/baz.py").not().eval(&output));
        assert!(contains("content baz.py").not().eval(&output));
    }

    #[test]
    fn test_include_folders() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--include=**/lowercase/**").assert().success();

        let output = env.read_output();
        debug!("Test include folders output:\n{}", output);
        assert!(contains("foo.py").eval(&output));
        assert!(contains("content foo.py").eval(&output));
        assert!(contains("baz.py").eval(&output));
        assert!(contains("content baz.py").eval(&output));
        assert!(contains("uppercase/FOO").not().eval(&output));
    }


    // #[test]
    // fn test_exclude_files() {
    //     let env = TestEnv::new();
    //     let mut cmd = env.command();
    //     cmd.arg("--exclude=**/foo.py,**/bar.py").assert().success();

    //     let output = env.read_output();
        // debug!("Test exclude files output:\n{}", output);
    //     assert!(contains("baz.py").eval(&output));
    //     assert!(contains("content baz.py").eval(&output));
    //     assert!(contains("foo.py").not().eval(&output));
    //     assert!(contains("content foo.py").not().eval(&output));
    //     assert!(contains("bar.py").not().eval(&output));  // `bar.py` isn't created in the test hierarchy
    //     assert!(contains("content bar.py").not().eval(&output));
    // }
    
    // #[test]
    // fn test_exclude_folders() {
    //     let env = TestEnv::new();
    //     let mut cmd = env.command();
    //     cmd.arg("--exclude=**/uppercase/**").assert().success();

    //     let output = env.read_output();
    //     debug!("Test exclude folders output:\n{}", output);
    //     assert!(contains("foo.py").eval(&output));
    //     assert!(contains("content foo.py").eval(&output));
    //     assert!(contains("baz.py").eval(&output));
    //     assert!(contains("content baz.py").eval(&output));
    //     assert!(contains("uppercase").not().eval(&output));
    // }

    // #[test]
    // fn test_include_exclude_combinations() {
    //     let env = TestEnv::new();
    //     let mut cmd = env.command();
    //     cmd.arg("--include=*.py,**/lowercase/**")
    //         .arg("--exclude=**/foo.py,**/uppercase/**")
    //         .arg("--conflict-include")
    //         .assert().success();

    //     let output = env.read_output();
    //     debug!("Test include and exclude combinations output:\n{}", output);
    //     assert!(contains("baz.py").eval(&output));
    //     assert!(contains("content baz.py").eval(&output));
    //     assert!(contains("foo.py").not().eval(&output));
    //     assert!(contains("content foo.py").not().eval(&output));
    //     assert!(contains("FOO.py").not().eval(&output));
    //     assert!(contains("CONTENT FOO.PY").not().eval(&output));
    // }

    // #[test]
    // fn test_no_filters() {
    //     let env = TestEnv::new();
    //     let mut cmd = env.command();
    //     cmd.assert().success();

    //     let output = env.read_output();
    //     debug!("Test no filters output:\n{}", output);
    //     assert!(contains("foo.py").eval(&output));
    //     assert!(contains("content foo.py").eval(&output));
    //     assert!(contains("baz.py").eval(&output));
    //     assert!(contains("content baz.py").eval(&output));
    //     assert!(contains("FOO.py").eval(&output));
    //     assert!(contains("CONTENT FOO.PY").eval(&output));
    //     assert!(contains("BAZ.py").eval(&output));
    //     assert!(contains("CONTENT BAZ.PY").eval(&output));
    // }
}
