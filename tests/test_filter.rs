use code2prompt::filter::should_include_file;
use colored::*;
use once_cell::sync::Lazy;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
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
    fn test_exclude_by_file_name() {
        let path = Path::new("/Users/test/code2prompt/output.txt");
        let include_patterns = vec![];
        let exclude_patterns = vec!["output.txt".to_string()];
        let include_priority = false;

        assert!(!should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
            include_priority
        ));
    }

    #[test]
    fn test_exclude_by_glob_pattern() {
        let path = Path::new("/Users/test/code2prompt/output.txt");
        let include_patterns = vec![];
        let exclude_patterns = vec!["**/output.txt".to_string()];
        let include_priority = false;

        assert!(!should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
            include_priority
        ));
    }

    #[test]
    fn test_include_by_file_name() {
        let path = Path::new("/Users/test/code2prompt/output.txt");
        let include_patterns = vec!["output.txt".to_string()];
        let exclude_patterns = vec![];
        let include_priority = true;

        assert!(should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
            include_priority
        ));
    }

    #[test]
    fn test_include_and_exclude_conflict() {
        let path = Path::new("/Users/test/code2prompt/output.txt");
        let include_patterns = vec!["output.txt".to_string()];
        let exclude_patterns = vec!["**/output.txt".to_string()];
        let include_priority = true;

        assert!(should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
            include_priority
        ));
    }

    #[test]
    fn test_no_include_no_exclude_path() {
        let path = Path::new("src/main.rs");
        let include_patterns: Vec<String> = vec![];
        let exclude_patterns: Vec<String> = vec![];
        let include_priority = false;
        assert!(should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
            include_priority
        ));
    }

    #[test]
    fn test_no_include_no_exclude_empty() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec![];
        let exclude_patterns = vec![];
        let include_priority = true;

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
                include_priority
            ));
        }
    }
    
    // ¬Include && Exclude
    #[test]
    fn test_no_include_exclude_path() {
        let path = Path::new("src/main.rs");
        let include_patterns: Vec<String> = vec![];
        let exclude_patterns: Vec<String> = vec!["*.rs".to_string()];
        let include_priority = false;
        assert!(!should_include_file(
            &path,
            &include_patterns,
            &exclude_patterns,
            include_priority
        ));
    }

    #[test]
    fn test_no_include_exclude_patterns() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec![];
        let exclude_patterns = vec!["*.txt".to_string()];
        let include_priority = false;

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
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }

    #[test]
    fn test_no_include_exclude_files() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec![];
        let exclude_patterns = vec!["**/foo.py".to_string(), "**/bar.py".to_string()];
        let include_priority = false;

        for file in ["lowercase/foo.py", "lowercase/bar.py"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }

    #[test]
    fn test_no_include_exclude_folders() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec![];
        let exclude_patterns = vec!["**/lowercase/**".to_string()];
        let include_priority = false;

        // Ces fichiers doivent être exclus
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/qux.txt",
        ] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

        // Ces fichiers doivent être inclus
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
                include_priority
            ));
        }
    }

    // Include && ¬Exclude
    #[test]
    fn test_include_no_exclude_patterns() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["*.py".to_string()];
        let exclude_patterns = vec![];
        let include_priority = false;

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
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }

    #[test]
    fn test_include_no_exclude_folders() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["**/lowercase/**".to_string()];
        let exclude_patterns = vec![];
        let include_priority = true;

        // Ces fichiers doivent être inclus
        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/qux.txt",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

        // Ces fichiers doivent être exclus
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
                include_priority
            ));
        }
    }

    #[test]
    fn test_include_no_exclude_files() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["**/foo.py".to_string(), "**/bar.py".to_string()];
        let exclude_patterns = vec![];
        let include_priority = false;

        for file in ["lowercase/foo.py", "lowercase/bar.py"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }

    // Include && Exclude && IncludePriority
    #[test]
    fn test_include_exclude_conflict_file() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["**/foo.py".to_string()];
        let exclude_patterns = vec!["**/foo.py".to_string()];
        let include_priority = true;

        for file in ["lowercase/foo.py"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }

    #[test]
    fn test_include_exclude_conflict_extension() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["*.py".to_string()];
        let exclude_patterns = vec!["*.py".to_string()];
        let include_priority = true;

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
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }

    #[test]
    fn test_include_exclude_conflict_folder() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["**/lowercase/**".to_string()];
        let exclude_patterns = vec!["**/lowercase/**".to_string()];
        let include_priority = true;

        for file in [
            "lowercase/foo.py",
            "lowercase/bar.py",
            "lowercase/qux.txt",
            "lowercase/baz.py",
            "lowercase/corge.txt",
            "lowercase/grault.txt",
        ] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

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
                include_priority
            ));
        }
    }
    
    #[test]
    fn test_include_exclude_priority_include() {
        let base_path = TEST_DIR.path();
    
        let include_patterns = vec!["**/*.py".to_string()];
        let exclude_patterns = vec!["**/uppercase/*".to_string()];
        let include_priority = true;
    
        // Ces fichiers doivent être inclus car priorité à l'inclusion
        for file in ["lowercase/foo.py", "uppercase/FOO.py"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }
    
        // Ces fichiers doivent être exclus
        for file in ["lowercase/qux.txt", "uppercase/QUX.txt", ".secret/secret.txt"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }
    }

    // Include && Exclude && ¬IncludePriority
    #[test]
    fn test_include_exclude_priority_exclude() {
        let base_path = TEST_DIR.path();

        let include_patterns = vec!["**/*.py".to_string()];
        let exclude_patterns = vec!["**/uppercase/*".to_string()];
        let include_priority = false;

        // Ces fichiers doivent être inclus
        for file in ["lowercase/foo.py"] {
            let path = base_path.join(file);
            assert!(should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }

        // Ces fichiers doivent être exclus car priorité à l'exclusion
        for file in ["uppercase/FOO.py", "uppercase/BAR.py",".secret/secret.txt"] {
            let path = base_path.join(file);
            assert!(!should_include_file(
                &path,
                &include_patterns,
                &exclude_patterns,
                include_priority
            ));
        }
    }

}
