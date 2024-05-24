// tests/test_filter.rs

use std::path::Path;
use std::fs::{self, File, read_to_string};
use std::io::Write;
use tempfile::tempdir;
use code2prompt::filter::should_include_file;

fn get_path(file_name: &str) -> &Path {
    Path::new(file_name)
}

fn create_temp_file(dir: &Path, name: &str, content: &str) {
    let file_path = dir.join(name);
    println!("Creating file: {:?}", file_path); // Debugging line
    let parent_dir = file_path.parent().unwrap();
    fs::create_dir_all(parent_dir).expect(&format!("Failed to create directory: {:?}", parent_dir));
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
        ("uppercase/BAZ.py", "CONTENT BAZ.PY"),
        ("uppercase/QUX.txt", "CONTENT QUX.TXT"),
        ("uppercase/CORGE.txt", "CONTENT CORGE.TXT"),
        ("uppercase/GRAULT.txt", "CONTENT GRAULT.TXT"),
    ];

    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }
}

#[test]
fn test_include_extensions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let include_extensions = vec!["py".to_string()];
    let exclude_extensions = vec![];
    let include_files = vec![];
    let exclude_files = vec![];

    for file in ["lowercase/foo.py", "lowercase/bar.py", "lowercase/baz.py", "uppercase/FOO.py", "uppercase/BAR.py", "uppercase/BAZ.py"] {
        let path = dir.path().join(file);
        assert!(should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }

    for file in ["lowercase/qux.txt", "lowercase/corge.txt", "lowercase/grault.txt", "uppercase/QUX.txt", "uppercase/CORGE.txt", "uppercase/GRAULT.txt"] {
        let path = dir.path().join(file);
        assert!(!should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }
}

#[test]
fn test_exclude_extensions() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let include_extensions = vec![];
    let exclude_extensions = vec!["txt".to_string()];
    let include_files = vec![];
    let exclude_files = vec![];

    for file in ["lowercase/foo.py", "lowercase/bar.py", "lowercase/baz.py", "uppercase/FOO.py", "uppercase/BAR.py", "uppercase/BAZ.py"] {
        let path = dir.path().join(file);
        assert!(should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }

    for file in ["lowercase/qux.txt", "lowercase/corge.txt", "lowercase/grault.txt", "uppercase/QUX.txt", "uppercase/CORGE.txt", "uppercase/GRAULT.txt"] {
        let path = dir.path().join(file);
        assert!(!should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }
}

#[test]
fn test_include_files() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let include_extensions = vec![];
    let exclude_extensions = vec![];
    let include_files = vec!["foo.py".to_string(), "bar.py".to_string()];
    let exclude_files = vec![];

    for file in ["lowercase/foo.py", "lowercase/bar.py", "uppercase/FOO.py", "uppercase/BAR.py"] {
        let path = dir.path().join(file);
        assert!(should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }

    for file in ["lowercase/baz.py", "lowercase/qux.txt", "lowercase/corge.txt", "lowercase/grault.txt", "uppercase/BAZ.py", "uppercase/QUX.txt", "uppercase/CORGE.txt", "uppercase/GRAULT.txt"] {
        let path = dir.path().join(file);
        assert!(!should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }
}

#[test]
fn test_exclude_files() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let include_extensions = vec![];
    let exclude_extensions = vec![];
    let include_files = vec![];
    let exclude_files = vec!["foo.py".to_string(), "bar.py".to_string()];

    for file in ["lowercase/foo.py", "lowercase/bar.py", "uppercase/FOO.py", "uppercase/BAR.py"] {
        let path = dir.path().join(file);
        assert!(!should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }

    for file in ["lowercase/baz.py", "lowercase/qux.txt", "lowercase/corge.txt", "lowercase/grault.txt", "uppercase/BAZ.py", "uppercase/QUX.txt", "uppercase/CORGE.txt", "uppercase/GRAULT.txt"] {
        let path = dir.path().join(file);
        assert!(should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }
}

#[test]
fn test_include_exclude_conflict_file() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let include_extensions = vec![];
    let exclude_extensions = vec![];
    let include_files = vec!["foo.py".to_string()];
    let exclude_files = vec!["foo.py".to_string()];

    for file in ["lowercase/foo.py", "uppercase/FOO.py"] {
        let path = dir.path().join(file);
        assert!(should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }

    for file in ["lowercase/bar.py", "lowercase/baz.py", "lowercase/qux.txt", "lowercase/corge.txt", "lowercase/grault.txt", "uppercase/BAR.py", "uppercase/BAZ.py", "uppercase/QUX.txt", "uppercase/CORGE.txt", "uppercase/GRAULT.txt"] {
        let path = dir.path().join(file);
        assert!(!should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }
}

#[test]
fn test_include_exclude_conflict_extension() {
    let dir = tempdir().unwrap();
    create_test_hierarchy(dir.path());

    let include_extensions = vec!["py".to_string()];
    let exclude_extensions = vec!["py".to_string()];
    let include_files = vec![];
    let exclude_files = vec![];

    for file in ["lowercase/foo.py", "lowercase/bar.py", "lowercase/baz.py", "uppercase/FOO.py", "uppercase/BAR.py", "uppercase/BAZ.py"] {
        let path = dir.path().join(file);
        assert!(should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }

    for file in ["lowercase/qux.txt", "lowercase/corge.txt", "lowercase/grault.txt", "uppercase/QUX.txt", "uppercase/CORGE.txt", "uppercase/GRAULT.txt"] {
        let path = dir.path().join(file);
        assert!(!should_include_file(&path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
    }
}
