use std::path::Path;
use glob::Pattern;
use colored::*;
use std::fs;

pub fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    conflict_include: bool,
) -> bool {
    // Convert the path to an absolute path and normalize it
    let canonical_path = fs::canonicalize(path).unwrap();
    let path_str = canonical_path.to_str().unwrap();

    let included = include_patterns.iter().any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));
    let excluded = exclude_patterns.iter().any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));

    println!(
        "Checking path: {:?}, {}: {}, {}: {}",
        path_str,
        "included".bold().green(),
        included,
        "excluded".bold().red(),
        excluded
    );

    match (included, excluded) {
        (true, true) => conflict_include,
        (true, false) => true,
        (false, true) => false,
        (false, false) => true,
    }
}
