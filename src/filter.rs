//! This module contains the logic for filtering files based on include and exclude patterns.

use colored::*;
use globset::{Glob, GlobSetBuilder, GlobSet};
use log::{info, warn, debug};
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
///
/// # Panics
///
/// * If the `GlobSetBuilder` fails to build the `GlobSet`, the function will panic.
fn build_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        // If the pattern does not contain a '/' or the platform’s separator, prepend "**/"
        let normalized_pattern = if !pattern.contains('/') && !pattern.contains(std::path::MAIN_SEPARATOR) {
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
    // Vérifie si on doit travailler avec des chemins relatifs
    let base_dir = std::env::current_dir().unwrap_or_else(|_| path.to_path_buf());
    let relative_path = path.strip_prefix(&base_dir).unwrap_or(path);

    // Construire les ensembles de motifs globaux
    let include_globset = build_globset(include_patterns);
    let exclude_globset = build_globset(exclude_patterns);

    // Vérification des correspondances
    let included = include_globset.is_match(relative_path) || include_globset.is_match(path);
    let excluded = exclude_globset.is_match(relative_path) || exclude_globset.is_match(path);

    // ~~~ Decision ~~~
    let result = match (included, excluded) {
        (true, true) => include_priority, // If both include and exclude patterns match, use the include_priority flag
        (true, false) => true,            // If the path is included and not excluded, include it
        (false, true) => false,           // If the path is excluded, exclude it
        (false, false) => include_patterns.is_empty(), // If no include patterns are provided, include everything
    };

    info!(
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
