use std::fs::{self, File, read_to_string};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::*;

fn create_temp_file(dir: &Path, name: &str, content: &str) {
    let file_path = dir.join(name);
    println!("Creating file: {:?}", file_path); // Debugging line
    let mut file = File::create(&file_path).expect(&format!("Failed to create temp file: {:?}", file_path));
    writeln!(file, "{}", content).expect(&format!("Failed to write to temp file: {:?}", file_path));
}

fn create_test_hierarchy(base_path: &Path) {
    let lowercase_dir = base_path.join("lowercase");
    let uppercase_dir = base_path.join("uppercase");

    fs::create_dir_all(&lowercase_dir).expect("Failed to create lowercase directory");
    fs::create_dir_all(&uppercase_dir).expect("Failed to create uppercase directory");

    let files = vec![
        ("lowercase/foo.py", "content foo.py"),
        ("lowercase/bar.py", "content bar.py"),
        ("lowercase/baz.py", "content baz.py"),
        ("lowercase/qux.txt", "content qux.txt"),
        ("lowercase/corge.txt", "content corge.txt"),
        ("lowercase/grault.txt", "content grault.txt"),
        ("uppercase/FOO.py", "CONTENT FOO.PY"),
        ("uppercase/BAR.py", "CONTENT BAR.PY"),
        ("uppercase/BAZ.TXT", "CONTENT BAZ.TXT"),
        ("uppercase/QUX.txt", "CONTENT QUX.TXT"),
        ("uppercase/CORGE.txt", "CONTENT CORGE.PY"),
        ("uppercase/GRAULT.txt", "CONTENT GRAULT.TXT"),
    ];

    for (file_path, content) in files {
        create_temp_file(&base_path.join(Path::new(file_path).parent().unwrap()), file_path, content);
    }
}

fn read_output_file(dir: &Path, file_name: &str) -> String {
    let file_path = dir.join(file_name);
    read_to_string(file_path).expect(&format!("Failed to read output file: {:?}", file_path))
}

#[test]
fn test_include_extensions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
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
    assert!(contains("qux.txt").not().eval(&output));
    assert!(contains("content qux.txt").not().eval(&output));
    assert!(contains("bar.py").eval(&output));
    assert!(contains("content bar.py").eval(&output));
}

#[test]
fn test_exclude_extensions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
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
    assert!(contains("bar.py").eval(&output));
    assert!(contains("content bar.py").eval(&output));
    assert!(contains("qux.txt").not().eval(&output));
    assert!(contains("content qux.txt").not().eval(&output));
}

#[test]
fn test_include_files() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg("--include-files=foo.py,bar.py")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include files output:\n{}", output);
    assert!(contains("foo.py").eval(&output));
    assert!(contains("content foo.py").eval(&output));
    assert!(contains("bar.py").eval(&output));
    assert!(contains("content bar.py").eval(&output));
    assert!(contains("baz.py").not().eval(&output));
    assert!(contains("content baz.py").not().eval(&output));
}

#[test]
fn test_exclude_files() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg("--exclude-files=foo.py,bar.py")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test exclude files output:\n{}", output);
    assert!(contains("baz.py").eval(&output));
    assert!(contains("content baz.py").eval(&output));
    assert!(contains("foo.py").not().eval(&output));
    assert!(contains("content foo.py").not().eval(&output));
    assert!(contains("bar.py").not().eval(&output));
    assert!(contains("content bar.py").not().eval(&output));
}

#[test]
fn test_include_folders() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg("--include-folders=lowercase")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include folders output:\n{}", output);
    assert!(contains("lowercase/foo.py").eval(&output));
    assert!(contains("lowercase/content foo.py").eval(&output));
    assert!(contains("lowercase/bar.py").eval(&output));
    assert!(contains("lowercase/content bar.py").eval(&output));
    assert!(contains("uppercase").not().eval(&output));
}

#[test]
fn test_exclude_folders() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg("--exclude-folders=lowercase")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test exclude folders output:\n{}", output);
    assert!(contains("uppercase/FOO.py").eval(&output));
    assert!(contains("uppercase/CONTENT FOO.PY").eval(&output));
    assert!(contains("uppercase/BAR.py").eval(&output));
    assert!(contains("uppercase/CONTENT BAR.PY").eval(&output));
    assert!(contains("lowercase").not().eval(&output));
    assert!(contains("lowercase/foo.py").not().eval(&output));
    assert!(contains("lowercase/content foo.py").not().eval(&output));
}

#[test]
fn test_include_folders_with_exclusions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg("--include-folders=lowercase")
        .arg("--exclude-files=foo.py")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test include folders with exclusions output:\n{}", output);
    assert!(contains("lowercase/bar.py").eval(&output));
    assert!(contains("lowercase/content bar.py").eval(&output));
    assert!(contains("lowercase/foo.py").not().eval(&output));
    assert!(contains("lowercase/content foo.py").not().eval(&output));
}

#[test]
fn test_exclude_folders_with_inclusions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let output_file = dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
    cmd.arg("--exclude-folders=lowercase")
        .arg("--include-files=lowercase/foo.py")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .arg("--no-clipboard")
        .arg(dir.path().to_str().unwrap());

    cmd.assert().success();

    let output = read_output_file(dir.path(), "output.txt");
    println!("Test exclude folders with inclusions output:\n{}", output);
    assert!(contains("lowercase/foo.py").eval(&output));
    assert!(contains("lowercase/content foo.py").eval(&output));
    assert!(contains("lowercase/bar.py").not().eval(&output));
    assert!(contains("lowercase/content bar.py").not().eval(&output));
    assert!(contains("uppercase/FOO.py").eval(&output));
    assert!(contains("uppercase/CONTENT FOO.PY").eval(&output));
}
