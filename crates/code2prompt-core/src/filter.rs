//! This module contains pure filtering logic for files based on glob patterns.
//!
//! This module provides reusable, stateless functions for pattern matching and file filtering.

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
///
/// This function is useful for implementing hierarchical inclusion/exclusion logic
/// where a rule applied to a parent directory should propagate to its descendants.
///
/// # Arguments
///
/// * `rel_path` - A relative path to check
/// * `set` - A set of paths to check against
///
/// # Returns
///
/// * `bool` - Returns `true` if the path or any of its ancestors is in the set
pub fn is_or_ancestor_in_set(rel_path: &Path, set: &HashSet<PathBuf>) -> bool {
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
///
/// This function implements a hierarchical decision process:
/// 1. Explicit excludes (self/ancestors) → false (highest priority, skip subtree for dirs)
/// 2. Explicit includes (self/ancestors) → true (second priority, include/recurse for dirs)
/// 3. Fallback: glob patterns (include if matches include or empty, unless excluded)
///
/// # Arguments
///
/// * `rel_path` - A relative path to check (must be relative to the root path)
/// * `include_globset` - A GlobSet specifying which files to include
/// * `exclude_globset` - A GlobSet specifying which files to exclude
/// * `explicit_includes` - A set of explicitly included paths (with ancestor propagation)
/// * `explicit_excludes` - A set of explicitly excluded paths (with ancestor propagation)
///
/// # Returns
///
/// * `bool` - Returns `true` if the path should be included/visited; otherwise, returns `false`
pub fn should_include_path(
    rel_path: &Path,
    include_globset: &GlobSet,
    exclude_globset: &GlobSet,
    explicit_includes: &HashSet<PathBuf>,
    explicit_excludes: &HashSet<PathBuf>,
) -> bool {
    // Step 1: Highest priority - explicit exclude on self or ancestor?
    if is_or_ancestor_in_set(rel_path, explicit_excludes) {
        debug!("Explicit exclude hit for: {:?}", rel_path);
        return false;
    }

    // Step 2: Explicit include on self or ancestor?
    if is_or_ancestor_in_set(rel_path, explicit_includes) {
        debug!("Explicit include hit for: {:?}", rel_path);
        return true;
    }

    // Step 3: Fallback to pure pattern matching logic
    let result = should_include_file(rel_path, include_globset, exclude_globset);

    debug!("Pattern fallback result={} for {:?}", result, rel_path);

    result
}
