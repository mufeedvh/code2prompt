//! This module contains the logic for filtering files based on include and exclude patterns.

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
fn build_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        // If the pattern does not contain a '/' or the platform’s separator, prepend "**/"
        let normalized_pattern =
            if !pattern.contains('/') && !pattern.contains(std::path::MAIN_SEPARATOR) {
                format!("**/{}", pattern)
            } else {
                pattern.clone()
            };

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
/// * `include_patterns` - A slice of glob pattern strings specifying which files to include.
///                        If empty, all files are considered included unless excluded.
/// * `exclude_patterns` - A slice of glob pattern strings specifying which files to exclude.
/// * `include_priority` - A boolean flag that, when set to `true`, gives include patterns
///                        precedence over exclude patterns in cases where both match.
///
/// # Returns
///
/// * `bool` - Returns `true` if the file should be included; otherwise, returns `false`.
pub fn should_include_file(
    path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
    include_priority: bool,
) -> bool {
    // ~~~ Initialization ~~~
    let include_globset = build_globset(include_patterns);
    let exclude_globset = build_globset(exclude_patterns);

    // ~~~ Matching ~~~
    let included = include_globset.is_match(path);
    let excluded = exclude_globset.is_match(path);

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => include_priority, // If both include and exclude patterns match, use the include_priority flag
        (true, false) => true,            // If the path is included and not excluded, include it
        (false, true) => false,           // If the path is excluded, exclude it
        (false, false) => include_patterns.is_empty(), // If no include patterns are provided, include everything
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
