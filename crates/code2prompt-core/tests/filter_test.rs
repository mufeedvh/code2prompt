/// This file tests the filter logic
/// Code2prompt uses the file globbing and globpattern to match files
/// Therefore you can match files:
use code2prompt_core::filter::{build_globset, should_include_file, should_include_path};
use colored::*;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
fn create_temp_file(dir: &Path, name: &str, content: &str) {
    let file_path = dir.join(name);
    let parent_dir = file_path.parent().unwrap();
    fs::create_dir_all(parent_dir).expect(&format!("Failed to create directory: {:?}", parent_dir));
    let mut file =
        File::create(&file_path).expect(&format!("Failed to create temp file: {:?}", file_path));
    writeln!(file, "{}", content).expect(&format!("Failed to write to temp file: {:?}", file_path));
}
static TEST_DIR: Lazy<TempDir> = Lazy::new(|| {
    let dir = tempdir().expect("Failed to create a temp directory");
    create_test_hierarchy(dir.path());
    dir
});
fn create_test_hierarchy(base_path: &Path) {
    let lowercase_dir = base_path.join("lowercase");
    let uppercase_dir = base_path.join("uppercase");
    let secret_dir = base_path.join(".secret");
    fs::create_dir_all(&lowercase_dir).expect("Failed to create lowercase directory");
    fs::create_dir_all(&uppercase_dir).expect("Failed to create uppercase directory");
    fs::create_dir_all(&secret_dir).expect("Failed to create secret directory");
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
        (".secret/secret.txt", "SECRET"),
    ];
    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Tempfiles created".green()
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    // ¬Include && ¬Exclude
    #[test]
    fn test_no_include_no_exclude_path() {
        let path = Path::new("src/main.rs");
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be included ~~~
        assert!(should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
        ));
    }
    #[test]
    fn test_no_include_no_exclude_empty() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be included ~~~
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    // ¬Include && Exclude
    #[test]
    fn test_no_include_exclude_path() {
        let path = Path::new("src/main.rs");
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec!["*.rs".to_string()]);
        assert!(!should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
        ));
    }
    #[test]
    /// Added for globset
    fn test_no_include_exclude_by_filename() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec!["default_template.hbs".to_string()]);
        // ~~~ Must be excluded ~~~
        let excluded_path = base_path.join("src/default_template.hbs");
        assert!(!should_include_file(
            &excluded_path,
            &include_patterns,
            &exclude_patterns,
        ));
    }
    #[test]
    fn test_no_include_exclude_path_patterns() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec!["lowercase/{*.txt,*.py}".to_string()]);
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
        ] {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(!should_include_file(
                &relative_path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be included ~~~
        for file in [
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ] {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(should_include_file(
                &relative_path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_no_include_exclude_patterns() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec!["*.txt".to_string()]);
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be included ~~~
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_no_include_exclude_files() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns =
            build_globset(&vec!["**/foo.py".to_string(), "**/bar.py".to_string()]);
        // ~~~ Must be excluded ~~~
        for file in ["lowercase/foo.py", "lowercase/bar.py"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be included ~~~
        for file in [
            "lowercase/baz.py",
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_no_include_exclude_folders() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec![]);
        let exclude_patterns = build_globset(&vec!["**/lowercase/**".to_string()]);
        // ~~~ Must be excluded ~~~
        for file in ["lowercase/foo.py", "lowercase/bar.py", "lowercase/qux.txt"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be included ~~~
        for file in [
            "uppercase/FOO.py",
            "uppercase/QUX.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    // Include && ¬Exclude
    #[test]
    fn test_include_no_exclude_patterns() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["*.py".to_string()]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be included ~~~
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    /// added for globset
    fn test_include_no_exclude_by_filename() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["default_template.hbs".to_string()]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be excluded ~~~
        for file in ["src/filter.rs", "src/git.rs", "src/lib.rs", "src/token.rs"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be included ~~~
        let include_path = base_path.join("src/default_template.hbs");
        assert!(should_include_file(
            &include_path,
            &include_patterns,
            &exclude_patterns,
        ));
    }
    #[test]
    fn test_include_no_exclude_by_path_pattern() {
        let base_path = TEST_DIR.path();
        // let include_patterns = vec!["lowercase/*.txt".to_string(), "lowercase/*.py".to_string()];
        let include_patterns = build_globset(&vec!["lowercase/{*.txt,*.py}".to_string()]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be included ~~~
        for file in [
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
        ] {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(should_include_file(
                &relative_path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ] {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(!should_include_file(
                &relative_path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_include_no_exclude_folders() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["**/lowercase/**".to_string()]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be included ~~~
        for file in ["lowercase/foo.py", "lowercase/bar.py", "lowercase/qux.txt"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "uppercase/FOO.py",
            "uppercase/QUX.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_include_no_exclude_files() {
        let base_path = TEST_DIR.path();
        let include_patterns =
            build_globset(&vec!["**/foo.py".to_string(), "**/bar.py".to_string()]);
        let exclude_patterns = build_globset(&vec![]);
        // ~~~ Must be included ~~~
        for file in ["lowercase/foo.py", "lowercase/bar.py"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/baz.py",
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    // Include && Exclude - Exclude takes precedence
    #[test]
    fn test_include_exclude_conflict_file() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["**/foo.py".to_string()]);
        let exclude_patterns = build_globset(&vec!["**/foo.py".to_string()]);
        // ~~~ Must be excluded (exclude takes precedence) ~~~
        for file in ["lowercase/foo.py"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/bar.py",
            "lowercase/baz.py",
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_include_exclude_conflict_extension() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["*.py".to_string()]);
        let exclude_patterns = build_globset(&vec!["*.py".to_string()]);
        // ~~~ Must be excluded (exclude takes precedence) ~~~
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_include_exclude_conflict_folder() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["**/lowercase/**".to_string()]);
        let exclude_patterns = build_globset(&vec!["**/lowercase/**".to_string()]);
        // ~~~ Must be excluded (exclude takes precedence) ~~~
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/qux.txt",
            "lowercase/baz.py",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }
    #[test]
    fn test_include_exclude_exclude_takes_precedence() {
        let base_path = TEST_DIR.path();
        let include_patterns = build_globset(&vec!["**/*.py".to_string()]);
        let exclude_patterns = build_globset(&vec!["**/uppercase/*".to_string()]);
        // ~~~ Must be included (not excluded) ~~~
        for file in ["lowercase/foo.py"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded (exclude takes precedence) ~~~
        for file in ["uppercase/FOO.py"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
        // ~~~ Must be excluded ~~~
        for file in [
            "lowercase/qux.txt",
            "uppercase/QUX.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
            ));
        }
    }

    #[test]
    fn test_brace_expansion_first_item() {
        let base_path = TEST_DIR.path();
        // This pattern uses brace expansion to match foo.py, bar.py, and baz.py
        // The issue was that the first item (foo.py) wasn't being considered
        let include_patterns = build_globset(&vec!["lowercase/{foo.py,bar.py,baz.py}".to_string()]);
        let exclude_patterns =
            build_globset(&vec!["lowercase/{qux.py,corge.py,grault.py}".to_string()]);
        // ALL files in the brace expansion should be included
        for file in ["foo.py", "bar.py", "baz.py"] {
            let path = base_path.join("lowercase").join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(
                should_include_file(&relative_path, &include_patterns, &exclude_patterns,),
                "Failed to include file: {}",
                file
            );
        }
        // Files not in the brace expansion should be excluded
        for file in ["qux.txt", "corge.txt", "grault.txt"] {
            let path = base_path.join("lowercase").join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(
                !should_include_file(&relative_path, &include_patterns, &exclude_patterns,),
                "Incorrectly included non-matching file: {}",
                file
            );
        }
    }

    #[test]
    fn test_brace_expansion_multiple_patterns() {
        let base_path = TEST_DIR.path();
        // Test with multiple patterns, each with brace expansion
        let include_patterns = build_globset(&vec![
            "lowercase/{foo,bar,baz}.py".to_string(),
            "uppercase/{FOO,BAR,BAZ}.py".to_string(),
        ]);
        let exclude_patterns = build_globset(&vec![]);
        // All files in the brace expansions should be included
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ] {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(
                should_include_file(&relative_path, &include_patterns, &exclude_patterns,),
                "Failed to include file: {}",
                file
            );
        }
        // Non-matching files should be excluded
        for file in [
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ] {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(
                !should_include_file(&relative_path, &include_patterns, &exclude_patterns,),
                "Incorrectly included non-matching file: {}",
                file
            );
        }
    }

    #[test]
    fn test_explicit_include_file_overrides_ancestor_exclude() {
        // Scenario: User runs `code2prompt *.rs`, unselects a folder (goes to explicit_excludes),
        // then selects a specific file in that folder (goes to explicit_includes).
        // The specific file should be included despite the ancestor being excluded.

        let mut explicit_includes = HashSet::new();
        let mut explicit_excludes = HashSet::new();

        // Folder is explicitly excluded
        explicit_excludes.insert(PathBuf::from("src"));

        // But a specific file in that folder is explicitly included
        explicit_includes.insert(PathBuf::from("src/main.rs"));

        let include_gs = build_globset(&vec!["*.rs".to_string()]);
        let exclude_gs = build_globset(&Vec::<String>::new());

        // The specific file should be included despite its parent being excluded
        assert!(
            should_include_path(
                &PathBuf::from("src/main.rs"),
                &include_gs,
                &exclude_gs,
                &explicit_includes,
                &explicit_excludes,
            ),
            "Explicit include of specific file should override ancestor exclude"
        );

        // But other files in the excluded folder should still be excluded
        assert!(
            !should_include_path(
                &PathBuf::from("src/lib.rs"),
                &include_gs,
                &exclude_gs,
                &explicit_includes,
                &explicit_excludes,
            ),
            "Other files in excluded folder should remain excluded"
        );
    }

    #[test]
    fn test_explicit_include_file_overrides_deeper_ancestor_exclude() {
        // Test with deeper nesting
        let mut explicit_includes = HashSet::new();
        let mut explicit_excludes = HashSet::new();

        // Parent folder is explicitly excluded
        explicit_excludes.insert(PathBuf::from("src/utils"));

        // But a specific file deep in that folder is explicitly included
        explicit_includes.insert(PathBuf::from("src/utils/helpers/important.rs"));

        let include_gs = build_globset(&vec!["*.rs".to_string()]);
        let exclude_gs = build_globset(&Vec::<String>::new());

        // The specific file should be included despite its ancestor being excluded
        assert!(
            should_include_path(
                &PathBuf::from("src/utils/helpers/important.rs"),
                &include_gs,
                &exclude_gs,
                &explicit_includes,
                &explicit_excludes,
            ),
            "Explicit include of specific file should override deeper ancestor exclude"
        );
    }

    #[test]
    fn test_explicit_exclude_file_still_overrides_explicit_include_same_file() {
        // This test ensures we don't break the existing behavior where
        // explicit exclude of the SAME file takes precedence over explicit include
        let mut explicit_includes = HashSet::new();
        let mut explicit_excludes = HashSet::new();

        // Same file is in both sets
        explicit_includes.insert(PathBuf::from("src/main.rs"));
        explicit_excludes.insert(PathBuf::from("src/main.rs"));

        let include_gs = build_globset(&Vec::<String>::new());
        let exclude_gs = build_globset(&Vec::<String>::new());

        // Explicit exclude should still win when it's the same file
        assert!(
            !should_include_path(
                &PathBuf::from("src/main.rs"),
                &include_gs,
                &exclude_gs,
                &explicit_includes,
                &explicit_excludes,
            ),
            "Explicit exclude of same file should still take precedence over explicit include"
        );
    }
}
