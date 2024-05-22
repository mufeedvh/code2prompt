use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::*;

fn create_temp_file(dir: &Path, name: &str, content: &str) {
    let file_path = dir.join(name);
    let mut file = File::create(file_path).unwrap();
    writeln!(file, "{}", content).unwrap();
}

#[test]
fn test_include_extensions() {
    let dir = tempdir().unwrap();
    create_temp_file(dir.path(), "test1.rs", "fn main() {}");
    create_temp_file(dir.path(), "test2.py", "print('Hello, world!')");
    create_temp_file(dir.path(), "test3.rs", "fn main() {}");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-extensions=rs")
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success().stdout(contains("test1.rs"));
    cmd.assert().success().stdout(contains("test3.rs"));
    cmd.assert().success().stdout(contains("fn main() {}"));
    cmd.assert().stdout(contains("test2.py").not());
    cmd.assert().stdout(contains("print('Hello, world!')").not());
}

#[test]
fn test_exclude_extensions() {
    let dir = tempdir().unwrap();
    create_temp_file(dir.path(), "test1.rs", "fn main() {}");
    create_temp_file(dir.path(), "test2.py", "print('Hello, world!')");
    create_temp_file(dir.path(), "test3.rs", "fn main() {}");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--exclude-extensions=py")
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success().stdout(contains("test1.rs"));
    cmd.assert().success().stdout(contains("test3.rs"));
    cmd.assert().success().stdout(contains("fn main() {}"));
    cmd.assert().stdout(contains("test2.py").not());
    cmd.assert().stdout(contains("print('Hello, world!')").not());
}

#[test]
fn test_include_files() {
    let dir = tempdir().unwrap();
    create_temp_file(dir.path(), "include.rs", "fn main() {}");
    create_temp_file(dir.path(), "exclude.rs", "fn exclude() {}");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-files=include.rs")
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success().stdout(contains("include.rs"));
    cmd.assert().success().stdout(contains("fn main() {}"));
    cmd.assert().stdout(contains("exclude.rs").not());
    cmd.assert().stdout(contains("fn exclude() {}").not());
}

#[test]
fn test_exclude_files() {
    let dir = tempdir().unwrap();
    create_temp_file(dir.path(), "include.rs", "fn main() {}");
    create_temp_file(dir.path(), "exclude.rs", "fn exclude() {}");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--exclude-files=exclude.rs")
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success().stdout(contains("include.rs"));
    cmd.assert().success().stdout(contains("fn main() {}"));
    cmd.assert().stdout(contains("exclude.rs").not());
    cmd.assert().stdout(contains("fn exclude() {}").not());
}

#[test]
fn test_include_folders() {
    let dir = tempdir().unwrap();
    let include_dir = dir.path().join("include_folder");
    let exclude_dir = dir.path().join("exclude_folder");
    fs::create_dir(&include_dir).unwrap();
    fs::create_dir(&exclude_dir).unwrap();

    create_temp_file(&include_dir, "include.rs", "fn main() {}");
    create_temp_file(&exclude_dir, "exclude.rs", "fn exclude() {}");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-folders=include_folder")
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success().stdout(contains("include.rs"));
    cmd.assert().success().stdout(contains("fn main() {}"));
    cmd.assert().stdout(contains("exclude.rs").not());
    cmd.assert().stdout(contains("fn exclude() {}").not());
}

#[test]
fn test_exclude_folders() {
    let dir = tempdir().unwrap();
    let include_dir = dir.path().join("include_folder");
    let exclude_dir = dir.path().join("exclude_folder");
    fs::create_dir(&include_dir).unwrap();
    fs::create_dir(&exclude_dir).unwrap();

    create_temp_file(&include_dir, "include.rs", "fn main() {}");
    create_temp_file(&exclude_dir, "exclude.rs", "fn exclude() {}");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--exclude-folders=exclude_folder")
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success().stdout(contains("include.rs"));
    cmd.assert().success().stdout(contains("fn main() {}"));
    cmd.assert().stdout(contains("exclude.rs").not());
    cmd.assert().stdout(contains("fn exclude() {}").not());
}
