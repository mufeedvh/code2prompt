//! This module contains pure filtering logic for files based on glob patterns.
//!
//! This module provides reusable, stateless functions for pattern matching and file filtering.

use bracoxide::explode;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, warn};
use std::path::Path;

/// FilterEngine encapsulates pattern-based file filtering logic.
/// This handles the base patterns (A, B in the A,A',B,B' system).
#[derive(Debug, Clone)]
pub struct FilterEngine {
    include_globset: GlobSet,
    exclude_globset: GlobSet,
}

impl FilterEngine {
    /// Create a new FilterEngine with the given patterns
    pub fn new(include_patterns: &[String], exclude_patterns: &[String]) -> Self {
        Self {
            include_globset: build_globset(include_patterns),
            exclude_globset: build_globset(exclude_patterns),
        }
    }

    /// Check if a file matches the base patterns (A, B logic)
    pub fn matches_patterns(&self, path: &Path) -> bool {
        should_include_file(path, &self.include_globset, &self.exclude_globset)
    }

    /// Get access to the include globset (for advanced usage)
    pub fn include_globset(&self) -> &GlobSet {
        &self.include_globset
    }

    /// Get access to the exclude globset (for advanced usage)
    pub fn exclude_globset(&self) -> &GlobSet {
        &self.exclude_globset
    }

    /// Check if there are any include patterns
    pub fn has_include_patterns(&self) -> bool {
        !self.include_globset.is_empty()
    }

    /// Check if a file is excluded by exclude patterns
    pub fn is_excluded(&self, path: &Path) -> bool {
        self.exclude_globset.is_match(path)
    }
}

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

    match builder.build() {
        Ok(set) => set,
        Err(e) => {
            warn!("❌ Failed to build GlobSet: {e}");
            GlobSetBuilder::new()
                .build()
                .expect("empty GlobSet never fails")
        }
    }
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
///   If empty, all files are considered included unless excluded.
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
