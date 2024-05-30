//! This module contains the logic for filtering files based on include and exclude patterns.

use colored::*;
use glob::Pattern;
use log::{debug, error};
use std::fs;
use std::path::Path;

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
    let canonical_path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to canonicalize path: {}", e);
            return false;
        }
    };
    let path_str = canonical_path.to_str().unwrap();

    // ~~~ Check glob patterns ~~~
    let included = include_patterns
        .iter()
        .any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));
    let excluded = exclude_patterns
        .iter()
        .any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));

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
