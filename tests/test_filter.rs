// tests/test_filter.rs

use std::path::Path;
use code2prompt::filter::should_include_file;

fn get_path(file_name: &str) -> &Path {
    Path::new(file_name)
}

#[test]
fn test_include_extensions() {
    let path = get_path("foo.py");
    let include_extensions = vec!["py".to_string()];
    let exclude_extensions = vec![];
    let include_files = vec![];
    let exclude_files = vec![];

    assert!(should_include_file(path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
}

#[test]
fn test_exclude_extensions() {
    let path = get_path("foo.txt");
    let include_extensions = vec![];
    let exclude_extensions = vec!["txt".to_string()];
    let include_files = vec![];
    let exclude_files = vec![];

    assert!(!should_include_file(path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
}

#[test]
fn test_include_files() {
    let path = get_path("foo.py");
    let include_extensions = vec![];
    let exclude_extensions = vec![];
    let include_files = vec!["foo.py".to_string()];
    let exclude_files = vec![];

    assert!(should_include_file(path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
}

#[test]
fn test_exclude_files() {
    let path = get_path("foo.py");
    let include_extensions = vec![];
    let exclude_extensions = vec![];
    let include_files = vec![];
    let exclude_files = vec!["foo.py".to_string()];

    assert!(!should_include_file(path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
}

#[test]
fn test_include_exclude_conflict_file() {
    let path = get_path("foo.py");
    let include_extensions = vec![];
    let exclude_extensions = vec![];
    let include_files = vec!["foo.py".to_string()];
    let exclude_files = vec!["foo.py".to_string()];

    assert!(should_include_file(path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
}

#[test]
fn test_include_exclude_conflict_extension() {
    let path = get_path("foo.py");
    let include_extensions = vec!["py".to_string()];
    let exclude_extensions = vec!["py".to_string()];
    let include_files = vec![];
    let exclude_files = vec![];

    assert!(should_include_file(path, &include_extensions, &exclude_extensions, &include_files, &exclude_files));
}
