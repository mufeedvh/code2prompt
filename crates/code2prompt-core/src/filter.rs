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
pub fn build_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    let expanded_patterns = if !patterns.is_empty() && patterns[0].contains("/{") {
        expand_brace_patterns(patterns)
    } else {
        patterns.to_vec()
    };

    for pattern in expanded_patterns {
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
    include_globset: &GlobSet,
    exclude_globset: &GlobSet,
    include_priority: bool,
) -> bool {
    // ~~~ Matching ~~~
    let included = include_globset.is_match(path);
    let excluded = exclude_globset.is_match(path);

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => include_priority, // If both include and exclude patterns match, use the include_priority flag
        (true, false) => true,            // If the path is included and not excluded, include it
        (false, true) => false,           // If the path is excluded, exclude it
        (false, false) => include_globset.is_empty(), // If no include patterns are provided, include everything
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

/// Expands glob patterns containing `{}` into multiple separate patterns.
///
/// This function detects patterns with brace expansion (e.g., `"src/{foo,bar}/**"`),
/// extracts the base prefix, and generates multiple patterns with expanded values.
///
/// # Arguments
///
/// * `patterns` - A slice of `String` containing glob patterns to be expanded.
///
/// # Returns
///
/// * A `Vec<String>` containing expanded patterns.
fn expand_brace_patterns(patterns: &[String]) -> Vec<String> {
    let joined_patterns = patterns.join(",");
    let brace_start_index = joined_patterns.find("/{").unwrap();
    let common_prefix = &joined_patterns[..brace_start_index];

    return joined_patterns[brace_start_index + 2..]
        .split(',')
        .map(|expanded_pattern| format!("{}/{}", common_prefix, expanded_pattern))
        .collect::<Vec<String>>();
}
