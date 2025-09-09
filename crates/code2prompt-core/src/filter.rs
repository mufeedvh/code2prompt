//! This module contains the logic for filtering files based on include and exclude patterns.

use crate::configuration::Code2PromptConfig;
use bracoxide::explode;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, warn};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
        // If the pattern does not contain a '/' or the platform's separator, prepend "**/"
        let normalized_pattern = if pattern.contains('/') {
            pattern.trim_start_matches("./").to_string()
        } else {
            format!("**/{}", pattern.trim_start_matches("./"))
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

/// Checks if the path or any ancestor is in the given set (for propagation).
fn is_or_ancestor_in_set(rel_path: &Path, set: &HashSet<PathBuf>) -> bool {
    let mut current = rel_path.to_path_buf();
    loop {
        if set.contains(&current) {
            return true;
        }
        if !current.pop() {
            // pop() returns false if now empty
            break;
        }
    }
    false
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

/// Determines whether to visit/include a path (file or dir) using layered logic.
/// - Explicit excludes (self/ancestors) → false (skip subtree for dirs).
/// - Explicit includes (self/ancestors) → true (include/recurse for dirs).
/// - Fallback: glob patterns (include if matches include or empty, unless excluded).
/// Note: rel_path must be relative to root.
pub fn should_visit_or_include(
    rel_path: &Path,
    config: &Code2PromptConfig,
    include_globset: &GlobSet,
    exclude_globset: &GlobSet,
) -> bool {
    // Step 1: Highest priority - explicit exclude on self or ancestor?
    if is_or_ancestor_in_set(rel_path, &config.explicit_excludes) {
        debug!("Explicit exclude hit for: {:?}", rel_path);
        return false;
    }

    // Step 2: Explicit include on self or ancestor?
    if is_or_ancestor_in_set(rel_path, &config.explicit_includes) {
        debug!("Explicit include hit for: {:?}", rel_path);
        return true;
    }

    // Step 3: Fallback to patterns (existing logic)
    let included = include_globset.is_match(rel_path);
    let excluded = exclude_globset.is_match(rel_path);
    let result = match (included, excluded) {
        (true, true) => false,
        (true, false) => true,
        (false, true) => false,
        (false, false) => include_globset.is_empty(),
    };

    debug!(
        "Pattern fallback: included={}, excluded={}, result={} for {:?}",
        included, excluded, result, rel_path
    );

    result
}
