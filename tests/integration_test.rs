use std::fs::{self, File, read_to_string};
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

fn create_test_hierarchy(base_path: &Path) {
    let lowercase_dir = base_path.join("lowercase");
    let uppercase_dir = base_path.join("uppercase");

    fs::create_dir_all(&lowercase_dir).unwrap();
    fs::create_dir_all(&uppercase_dir).unwrap();

    let files = vec![
        ("lowercase/foo.py", "content foo.py"),
        ("lowercase/bar.txt", "content bar.txt"),
        ("lowercase/baz.txt", "content baz.txt"),
        ("lowercase/qux.py", "content qux.py"),
        ("lowercase/corge.py", "content corge.py"),
        ("lowercase/grault.txt", "content grault.txt"),
        ("uppercase/FOO.py", "CONTENT FOO.PY"),
        ("uppercase/BAR.py", "CONTENT BAR.PY"),
        ("uppercase/BAZ.txt", "CONTENT BAZ.TXT"),
        ("uppercase/QUX.txt", "CONTENT QUX.TXT"),
        ("uppercase/CORGE.py", "CONTENT CORGE.PY"),
        ("uppercase/GRAULT.txt", "CONTENT GRAULT.TXT"),
    ];

    for (file_path, content) in files {
        create_temp_file(&base_path.join(file_path), file_path, content);
    }
}

fn read_output_file(dir: &Path, file_name: &str) -> String {
    let file_path = dir.join(file_name);
    read_to_string(file_path).unwrap()
}

#[test]
fn test_include_extensions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-extensions=py")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include extensions output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("qux.py").eval(&output));
    assert!(contains("content qux.py").eval(&output));
    assert!(contains("bar.txt").not().eval(&output));
    assert!(contains("content bar.txt").not().eval(&output));
}

#[test]
fn test_exclude_extensions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--exclude-extensions=txt")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test exclude extensions output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.txt").not().eval(&output));
    assert!(contains("content bar.txt").not().eval(&output));
}

#[test]
fn test_include_files() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-files=foo.py,bar.txt")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include files output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.txt").eval(&output));
    assert!(contains("content bar.txt").eval(&output));
    assert!(contains("baz.txt").not().eval(&output));
    assert!(contains("content baz.txt").not().eval(&output));
}

#[test]
fn test_exclude_files() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--exclude-files=foo.py,bar.txt")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test exclude files output:\n{}", output);
    assert!(contains("baz.txt").eval(&output));
    assert!(contains("content baz.txt").eval(&output));
    assert!(contains("foo.py").not().eval(&output));
    assert!(contains("content foo.py").not().eval(&output));
    assert!(contains("bar.txt").not().eval(&output));
    assert!(contains("content bar.txt").not().eval(&output));
}

#[test]
fn test_include_folders() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-folders=lowercase")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include folders output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.txt").eval(&output));
    assert!(contains("content bar.txt").eval(&output));
    assert!(contains("uppercase").not().eval(&output));
}

#[test]
fn test_exclude_folders() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--exclude-folders=uppercase")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test exclude folders output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.txt").eval(&output));
    assert!(contains("content bar.txt").eval(&output));
    assert!(contains("uppercase").not().eval(&output));
}

#[test]
fn test_include_exclude_combinations() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--include-extensions=py")
        .arg("--exclude-files=foo.py")
        .arg("--include-folders=lowercase")
        .arg("--exclude-folders=uppercase")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include and exclude combinations output:\n{}", output);
    assert!(contains("qux.py").eval(&output));
    assert!(contains("content qux.py").eval(&output));
    assert!(contains("foo.py").not().eval(&output));
    assert!(contains("content foo.py").not().eval(&output));
    assert!(contains("bar.txt").not().eval(&output));
    assert!(contains("content bar.txt").not().eval(&output));
}

#[test]
fn test_no_filters() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").unwrap();
    cmd.arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test no filters output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.txt").eval(&output));
    assert!(contains("content bar.txt").eval(&output));
    assert!(contains("FOO.py").eval(&output));
    assert!(contains("CONTENT FOO.PY").eval(&output));
    assert!(contains("BAZ.txt").eval(&output));
    assert!(contains("CONTENT BAZ.TXT").eval(&output));
}
