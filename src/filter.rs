//! This module contains the logic for filtering files based on include and exclude patterns.
use std::path::Path;
use glob::Pattern;
use colored::*;
use std::fs;
use log::{debug, error};

pub fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    conflict_include: bool,
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
    let included = include_patterns.iter().any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));
    let excluded = exclude_patterns.iter().any(|pattern| Pattern::new(pattern).unwrap().matches(path_str));

    debug!(
        "Checking path: {:?}, {}: {}, {}: {}",
        path_str,
        "included".bold().green(),
        included,
        "excluded".bold().red(),
        excluded
    );

    // ~~~ Decision ~~~
    match (included, excluded) {
        (true, true) => conflict_include, // If both include and exclude patterns match, use the conflict_include flag
        (true, false) => true, // If the path is included and not excluded, include it
        (false, true) => false, // If the path is excluded, exclude it
        (false, false) => include_patterns.is_empty(), // If no include patterns are provided, include everything
    }
}
