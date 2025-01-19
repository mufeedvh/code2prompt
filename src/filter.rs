//! This module contains the logic for filtering files based on include and exclude patterns.

use colored::*;
use glob::Pattern;
use log::{debug, error};
use std::fs;
use std::path::Path;

/// Helper function to check if a path matches any pattern
fn matches_any_pattern(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_str().unwrap_or("");

    patterns.iter().any(|pattern| {
        // Extract the file name
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        // Normalize the pattern to an absolute path
        let absolute_pattern = fs::canonicalize(Path::new(pattern)).unwrap_or_else(|_| Path::new(pattern).to_path_buf());
        // Match either the file name or the full path
        pattern == file_name || Pattern::new(absolute_pattern.to_str().unwrap_or(pattern)).unwrap().matches(path_str)
    })
}

/// Determines whether a file should be included based on include and exclude patterns.
///
/// # Arguments
///
/// * `path` - The path to the file to be checked.
/// * `include_patterns` - A slice of strings representing the include patterns.
/// * `exclude_patterns` - A slice of strings representing the exclude patterns.
/// * `include_priority` - A boolean indicating whether to give priority to include patterns if both include and exclude patterns match.
///
/// # Returns
///
/// * `bool` - `true` if the file should be included, `false` otherwise.
pub fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    include_priority: bool,
) -> bool {
    // ~~~ Clean path ~~~
    let canonical_path = fs::canonicalize(path).unwrap_or_else(|e| {
        error!("Failed to canonicalize path {:?}: {}", path, e);
        path.to_path_buf() // Fall back to the original path if canonicalization fails
    });
    let path_str = canonical_path.to_str().unwrap_or("");

    // ~~~ Check glob patterns ~~~
    let included = matches_any_pattern(&canonical_path, include_patterns);
    let excluded = matches_any_pattern(&canonical_path, exclude_patterns);

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => include_priority, // If both include and exclude patterns match, use the include_priority flag
        (true, false) => true,            // If the path is included and not excluded, include it
        (false, true) => false,           // If the path is excluded, exclude it
        (false, false) => include_patterns.is_empty(), // If no include patterns are provided, include everything
    };

    debug!(
        "Checking path: {:?}, {}: {}, {}: {}, decision: {}",
        path_str,
        "included".bold().green(),
        included,
        "excluded".bold().red(),
        excluded,
        result
    );
    result
}
