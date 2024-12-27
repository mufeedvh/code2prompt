//! This module contains the logic for filtering files based on include and exclude patterns.

use colored::*;
use glob::Pattern;
use log::{debug, error};
use std::fs;
use std::path::Path;
use std::io::Read;

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
/// Checks if a file's content contains the specified text
///
/// # Arguments
///
/// * `path` - The path to the file to check
/// * `text` - The text to search for in the file
///
/// # Returns
///
/// * `bool` - `true` if the text is found, `false` otherwise
fn file_contains_text(path: &Path, text: &str) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return false;
    }
    
    contents.contains(text)
}

pub fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    include_priority: bool,
    text_filter: Option<&str>,
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

    // ~~~ Check text filter ~~~
    let text_match = match text_filter {
        Some(text) => file_contains_text(path, text),
        None => true,
    };

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => include_priority && text_match, // If both include and exclude patterns match, use the include_priority flag
        (true, false) => text_match,                   // If the path is included and not excluded, include it if text matches
        (false, true) => false,                        // If the path is excluded, exclude it
        (false, false) => include_patterns.is_empty() && text_match, // If no include patterns are provided, include everything if text matches
    };

    debug!(
        "Checking path: {:?}, {}: {}, {}: {}, {}: {}, decision: {}",
        path_str,
        "included".bold().green(),
        included,
        "excluded".bold().red(),
        excluded,
        "text_match".bold().blue(),
        text_match,
        result
    );
    result
}
