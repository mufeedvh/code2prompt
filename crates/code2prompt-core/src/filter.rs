//! This module contains the logic for filtering files based on include and exclude patterns.

use bracoxide::explode;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, warn};
use std::path::Path;

/// Constructs a `GlobSet` from a list of glob patterns.
///
/// This function takes a slice of `String` patterns, attempts to convert each
/// pattern into a `Glob`, and adds it to a `GlobSetBuilder`. If any pattern is
/// invalid, it is ignored. The function then builds and returns a `GlobSet`.
///
/// # Arguments
///
/// * `patterns` - A slice of `String` containing glob patterns.
///
/// # Returns
///
/// * A `globset::GlobSet` containing all valid glob patterns from the input.
pub fn build_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    let mut expanded_patterns = Vec::new();
    for pattern in patterns {
        if pattern.contains('{') {
            match explode(pattern) {
                Ok(exp) => expanded_patterns.extend(exp),
                Err(e) => warn!("⚠️ Invalid brace pattern '{}': {:?}", pattern, e),
            }
        } else {
            expanded_patterns.push(pattern.clone());
        }
    }

    for pattern in expanded_patterns {
        // If the pattern does not contain a '/' or the platform’s separator, prepend "**/"
        let normalized_pattern = format!("**/{}", pattern.trim_start_matches("./"));

        match Glob::new(&normalized_pattern) {
            Ok(glob) => {
                builder.add(glob);
                debug!("✅ Glob pattern added: '{}'", normalized_pattern);
            }
            Err(_) => {
                warn!("⚠️ Invalid pattern: '{}'", normalized_pattern);
            }
        }
    }

    builder.build().expect("❌ Impossible to build GlobSet")
}

/// Determines whether a file should be included based on the provided glob patterns.
///
/// Note: The `path` argument must be a relative path (i.e. relative to the base directory)
/// for the patterns to match as expected. Absolute paths will not yield correct matching.
///
/// # Arguments
///
/// * `path` - A relative path to the file that will be checked against the patterns.
/// * `include_globset` - A GlobSet specifying which files to include.
///                       If empty, all files are considered included unless excluded.
/// * `exclude_globset` - A GlobSet specifying which files to exclude.
///
/// # Returns
///
/// * `bool` - Returns `true` if the file should be included; otherwise, returns `false`.
///
/// # Behavior
///
/// When both include and exclude patterns match, exclude patterns take precedence.
pub fn should_include_file(
    path: &Path,
    include_globset: &GlobSet,
    exclude_globset: &GlobSet,
) -> bool {
    // ~~~ Matching ~~~
    let included = include_globset.is_match(path);
    let excluded = exclude_globset.is_match(path);

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => false,  // If both match, exclude takes precedence
        (true, false) => true,  // If only included, include it
        (false, true) => false, // If only excluded, exclude it
        (false, false) => include_globset.is_empty(), // If no include patterns, include everything
    };

    debug!(
        "Result: {}, {}: {}, {}: {}, Path: {:?}",
        result,
        "included".bold().green(),
        included,
        "excluded".bold().red(),
        excluded,
        path.display()
    );
    result
}
