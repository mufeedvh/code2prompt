/// This file tests the filter logic
/// Code2prompt uses the file globbing and globpattern to match files
use code2prompt_core::filter::{build_globset, should_include_file};
use rstest::*;
use std::path::Path;
use tempfile::{TempDir, tempdir};

// ~~~ Fixtures ~~~
#[fixture]
fn test_dir() -> TempDir {
    let dir = tempdir().expect("Failed to create temp dir");
    let lowercase_dir = dir.path().join("lowercase");
    let uppercase_dir = dir.path().join("uppercase");
    let secret_dir = dir.path().join(".secret");
    std::fs::create_dir_all(&lowercase_dir).expect("Failed to create lowercase directory");
    std::fs::create_dir_all(&uppercase_dir).expect("Failed to create uppercase directory");
    std::fs::create_dir_all(&secret_dir).expect("Failed to create secret directory");

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
        let path = dir.path().join(file_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    dir
}

fn base_path(test_dir: &TempDir) -> &Path {
    test_dir.path()
}

// ~~~ Filter Tests ~~~
#[cfg(test)]
mod tests {
    use super::*;

    // Helper
    fn test_files_inclusion(
        base_path: &Path,
        include_patterns: &[String],
        exclude_patterns: &[String],
        expected_included: &[&str],
        expected_excluded: &[&str],
    ) {
        let include_globset = build_globset(include_patterns);
        let exclude_globset = build_globset(exclude_patterns);

        for file in expected_included {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(
                should_include_file(relative_path, &include_globset, &exclude_globset),
                "File {} should be included",
                file
            );
        }

        for file in expected_excluded {
            let path = base_path.join(file);
            let relative_path = path.strip_prefix(base_path).unwrap();
            assert!(
                !should_include_file(relative_path, &include_globset, &exclude_globset),
                "File {} should be excluded",
                file
            );
        }
    }

    // ~~~ No Pattern ~~~
    #[rstest]
    fn test_no_include_no_exclude_path() {
        let path = Path::new("src/main.rs");
        let include_patterns = build_globset(&[]);
        let exclude_patterns = build_globset(&[]);
        assert!(should_include_file(
            path,
            &include_patterns,
            &exclude_patterns
        ));
    }

    #[rstest]
    fn test_no_include_no_exclude_empty(test_dir: TempDir) {
        let base_path = base_path(&test_dir);
        let include_patterns = vec![];
        let exclude_patterns = vec![];

        let expected_included = &[
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
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            &[],
        );
    }

    // ~~~ Exclusion Only ~~~
    #[rstest]
    fn test_no_include_exclude_path() {
        let path = Path::new("src/main.rs");
        let include_patterns = build_globset(&[]);
        let exclude_patterns = build_globset(&["*.rs".to_string()]);
        assert!(!should_include_file(
            path,
            &include_patterns,
            &exclude_patterns
        ));
    }

    #[rstest]
    fn test_no_include_exclude_by_filename(test_dir: TempDir) {
        let base_path = base_path(&test_dir);
        let include_patterns = vec![];
        let exclude_patterns = vec!["default_template.hbs".to_string()];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            &[],
            &["src/default_template.hbs"],
        );
    }

    #[rstest]
    fn test_no_include_exclude_path_patterns(test_dir: TempDir) {
        let base_path = base_path(&test_dir);
        let include_patterns = vec![];
        let exclude_patterns = vec!["lowercase/{*.txt,*.py}".to_string()];

        let expected_included = &[
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ];

        let expected_excluded = &[
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_no_include_exclude_folders(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns: Vec<String> = vec![]; // include everything by default
        let exclude_patterns = vec!["**/lowercase/**".to_string()];

        let expected_included = &[
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ];

        let expected_excluded = &[
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_no_include_exclude_files(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns: Vec<String> = vec![]; // include everything by default
        let exclude_patterns = vec!["**/foo.py".to_string(), "**/bar.py".to_string()];

        let expected_included = &[
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
        ];

        let expected_excluded = &["lowercase/foo.py", "lowercase/bar.py"];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_no_include_exclude_patterns(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns: Vec<String> = vec![]; // include everything by default
        let exclude_patterns = vec!["*.txt".to_string()];

        let expected_included = &[
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ];

        let expected_excluded = &[
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    // ~~~ Inclusion Only ~~~
    #[rstest]
    fn test_include_no_exclude_patterns(test_dir: TempDir) {
        let base_path = base_path(&test_dir);
        let include_patterns = vec!["*.py".to_string()];
        let exclude_patterns = vec![];

        let expected_included = &[
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ];

        let expected_excluded = &[
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_no_exclude_files(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["**/foo.py".to_string(), "**/bar.py".to_string()];
        let exclude_patterns = vec![];

        let expected_included = &["lowercase/foo.py", "lowercase/bar.py"];

        let expected_excluded = &[
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
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_no_exclude_folders(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["**/lowercase/**".to_string()];
        let exclude_patterns = vec![];

        let expected_included = &[
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
        ];

        let expected_excluded = &[
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_no_exclude_by_path_pattern(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["lowercase/{*.txt,*.py}".to_string()];
        let exclude_patterns = vec![];

        let expected_included = &[
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
        ];

        let expected_excluded = &[
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            ".secret/secret.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_no_exclude_by_filename(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["default_template.hbs".to_string()];
        let exclude_patterns = vec![];

        let expected_included = &["src/default_template.hbs"];

        let expected_excluded = &["src/filter.rs", "src/git.rs", "src/lib.rs", "src/token.rs"];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    // ~~~ Inclusion & Exclusion ~~~
    #[rstest]
    fn test_include_exclude_conflict_file(test_dir: TempDir) {
        let base_path = base_path(&test_dir);
        let include_patterns = vec!["**/foo.py".to_string()];
        let exclude_patterns = vec!["**/foo.py".to_string()];

        // Tous les fichiers devraient être exclus (conflit, exclude l'emporte)
        let expected_excluded = &[
            "lowercase/foo.py",
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
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            &[],
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_exclude_exclude_takes_precedence(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["**/*.py".to_string()];
        let exclude_patterns = vec!["**/uppercase/*".to_string()];

        let expected_included = &["lowercase/foo.py", "lowercase/bar.py", "lowercase/baz.py"];

        let expected_excluded = &[
            "uppercase/FOO.py",   // excluded explicitly
            "lowercase/qux.txt",  // doesn’t match include
            "uppercase/QUX.txt",  // excluded explicitly
            ".secret/secret.txt", // doesn’t match include
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_exclude_conflict_folder(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["**/lowercase/**".to_string()];
        let exclude_patterns = vec!["**/lowercase/**".to_string()];

        let expected_included: &[&str] = &[]; // nothing should be included

        let expected_excluded = &[
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/qux.txt",
            "lowercase/baz.py",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    #[rstest]
    fn test_include_exclude_conflict_extension(test_dir: TempDir) {
        let base_path = test_dir.path();

        let include_patterns = vec!["*.py".to_string()];
        let exclude_patterns = vec!["*.py".to_string()];

        let expected_included: &[&str] = &[]; // nothing included

        let expected_excluded = &[
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
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }

    // ~~~ Brace expansion ~~~
    #[rstest]
    fn test_brace_expansion_first_item(test_dir: TempDir) {
        let base_path: &Path = base_path(&test_dir);
        let include_patterns = vec!["lowercase/{foo.py,bar.py,baz.py}".to_string()];
        let exclude_patterns = vec!["lowercase/{qux.py,corge.py,grault.py}".to_string()];

        let expected_included = &["foo.py", "bar.py", "baz.py"]
            .iter()
            .map(|f| format!("lowercase/{}", f))
            .collect::<Vec<_>>();

        let expected_excluded = &["qux.txt", "corge.txt", "grault.txt"]
            .iter()
            .map(|f| format!("lowercase/{}", f))
            .collect::<Vec<_>>();

        // Conversion pour utiliser avec test_files_inclusion
        let expected_included: Vec<&str> = expected_included.iter().map(|s| s.as_str()).collect();
        let expected_excluded: Vec<&str> = expected_excluded.iter().map(|s| s.as_str()).collect();

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            &expected_included,
            &expected_excluded,
        );
    }

    #[rstest]
    fn test_brace_expansion_multiple_patterns(test_dir: TempDir) {
        let base_path: &Path = base_path(&test_dir);

        let include_patterns = vec![
            "lowercase/{foo,bar,baz}.py".to_string(),
            "uppercase/{FOO,BAR,BAZ}.py".to_string(),
        ];
        let exclude_patterns = vec![];

        // Explicitly list what should be included
        let expected_included = &[
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/baz.py",
            "uppercase/FOO.py",
            "uppercase/BAR.py",
            "uppercase/BAZ.py",
        ];

        // Explicitly list what should be excluded
        let expected_excluded = &[
            "lowercase/qux.txt",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
            "uppercase/QUX.txt",
            "uppercase/CORGE.txt",
            "uppercase/GRAULT.txt",
            ".secret/secret.txt",
        ];

        test_files_inclusion(
            base_path,
            &include_patterns,
            &exclude_patterns,
            expected_included,
            expected_excluded,
        );
    }
}
