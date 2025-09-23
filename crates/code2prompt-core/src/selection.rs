//! This module contains the SelectionEngine that handles user file selection with precedence rules.
//!
//! The SelectionEngine implements the A,A',B,B' system where:
//! - A, B: Base patterns (handled by FilterEngine)
//! - A', B': User actions with precedence rules (specific > generic, recent > old)

use crate::filter::FilterEngine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Represents a user action on a file or directory
#[derive(Debug, Clone)]
pub struct SelectionAction {
    pub path: PathBuf,
    pub action: ActionType,
    pub timestamp: SystemTime,
    pub specificity: u32, // Higher = more specific (more path components)
}

/// Type of selection action
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    Include,
    Exclude,
}

/// SelectionEngine handles both pattern-based filtering and user actions
/// with clear precedence rules: specific > generic, recent > old
#[derive(Clone)]
pub struct SelectionEngine {
    /// Base pattern filtering (A, B in A,A',B,B' system)
    filter_engine: FilterEngine,

    /// User actions (A', B' in A,A',B,B' system)
    user_actions: Vec<SelectionAction>,

    /// Cache for performance
    cache: HashMap<PathBuf, bool>,
}

impl SelectionEngine {
    /// Create a new SelectionEngine with base patterns
    pub fn new(include_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Self {
        Self {
            filter_engine: FilterEngine::new(&include_patterns, &exclude_patterns),
            user_actions: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// The core decision method: determines if a file should be selected
    /// Uses precedence rules: specific > generic, recent > old
    pub fn is_selected(&mut self, path: &Path) -> bool {
        // Check cache first for performance
        if let Some(&cached) = self.cache.get(path) {
            return cached;
        }

        let result = self.compute_selection(path);
        self.cache.insert(path.to_path_buf(), result);
        result
    }

    /// Compute selection without caching
    fn compute_selection(&self, path: &Path) -> bool {
        // Rule 1: Find the most specific and recent user action
        if let Some(action) = self.find_applicable_user_action(path) {
            return action.action == ActionType::Include;
        }

        // Rule 2: Fall back to existing FilterEngine logic (A, B)
        if self.filter_engine.has_include_patterns() {
            // If there are include patterns, use them
            self.filter_engine.matches_patterns(path)
        } else {
            // No include patterns: default behavior is to include all files
            // (unless excluded by exclude patterns)
            !self.filter_engine.is_excluded(path)
        }
    }

    /// Find the most applicable user action using precedence rules
    fn find_applicable_user_action(&self, path: &Path) -> Option<&SelectionAction> {
        let applicable_actions: Vec<&SelectionAction> = self
            .user_actions
            .iter()
            .filter(|action| self.action_applies_to_path(action, path))
            .collect();

        if applicable_actions.is_empty() {
            return None;
        }

        // Apply precedence rules: specific > generic, recent > old
        applicable_actions.into_iter().max_by(|a, b| {
            // First compare specificity (higher is better)
            match a.specificity.cmp(&b.specificity) {
                std::cmp::Ordering::Equal => {
                    // If same specificity, compare timestamp (more recent is better)
                    a.timestamp.cmp(&b.timestamp)
                }
                other => other,
            }
        })
    }

    /// Check if a user action applies to a given path
    fn action_applies_to_path(&self, action: &SelectionAction, path: &Path) -> bool {
        // Exact match
        if action.path == path {
            return true;
        }

        // Directory action applies to all children
        if path.starts_with(&action.path) {
            return true;
        }

        false
    }

    /// Calculate specificity score for a path (more components = more specific)
    fn calculate_specificity(&self, path: &Path) -> u32 {
        path.components().count() as u32
    }

    /// User interaction: include a file or directory
    pub fn include_file(&mut self, path: PathBuf) {
        self.add_user_action(path, ActionType::Include);
    }

    /// User interaction: exclude a file or directory
    pub fn exclude_file(&mut self, path: PathBuf) {
        self.add_user_action(path, ActionType::Exclude);
    }

    /// User interaction: toggle selection state
    pub fn toggle_file(&mut self, path: PathBuf) {
        let current_state = self.is_selected(&path);
        let new_action = if current_state {
            ActionType::Exclude
        } else {
            ActionType::Include
        };
        self.add_user_action(path, new_action);
    }

    /// Add a user action with timestamp and specificity
    fn add_user_action(&mut self, path: PathBuf, action: ActionType) {
        let specificity = self.calculate_specificity(&path);
        let user_action = SelectionAction {
            path,
            action,
            timestamp: SystemTime::now(),
            specificity,
        };

        self.user_actions.push(user_action);
        self.cache.clear(); // Invalidate cache when actions change
    }

    /// Get all currently selected files by scanning the filesystem
    pub fn get_selected_files(&mut self, root_path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        // If we have user actions, return files based on those actions
        if !self.user_actions.is_empty() {
            let mut selected = Vec::new();

            // Clone the actions to avoid borrow checker issues
            let actions = self.user_actions.clone();

            // Collect files from user actions that are includes
            for action in &actions {
                if action.action == ActionType::Include {
                    // Check if this action is still the winning action for this path
                    if self.is_selected(&action.path) {
                        selected.push(action.path.clone());
                    }
                }
            }

            // Remove duplicates and sort
            selected.sort();
            selected.dedup();
            return Ok(selected);
        }

        // Otherwise, scan filesystem for pattern matches
        let mut selected = Vec::new();
        self.collect_selected_files_recursive(root_path, root_path, &mut selected)?;
        Ok(selected)
    }

    /// Recursively collect selected files
    fn collect_selected_files_recursive(
        &mut self,
        root_path: &Path,
        current_dir: &Path,
        selected: &mut Vec<PathBuf>,
    ) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Convert to relative path for selection checking
            let relative_path = if let Ok(rel) = path.strip_prefix(root_path) {
                rel
            } else {
                continue;
            };

            if self.is_selected(relative_path) {
                if path.is_file() {
                    selected.push(relative_path.to_path_buf());
                } else if path.is_dir() {
                    // Recursively check subdirectories
                    self.collect_selected_files_recursive(root_path, &path, selected)?;
                }
            }
        }
        Ok(())
    }

    /// Clear all user actions (reset to pattern-only behavior)
    pub fn clear_user_actions(&mut self) {
        self.user_actions.clear();
        self.cache.clear();
    }

    /// Get the number of user actions
    pub fn user_action_count(&self) -> usize {
        self.user_actions.len()
    }

    /// Check if there are any user actions
    pub fn has_user_actions(&self) -> bool {
        !self.user_actions.is_empty()
    }

    /// Get access to the underlying filter engine
    pub fn filter_engine(&self) -> &FilterEngine {
        &self.filter_engine
    }
}

impl std::fmt::Debug for SelectionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionEngine")
            .field("filter_engine", &self.filter_engine)
            .field("user_actions", &self.user_actions)
            .field("cache_size", &self.cache.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity_calculation() {
        let engine = SelectionEngine::new(vec![], vec![]);

        assert_eq!(engine.calculate_specificity(Path::new("file.rs")), 1);
        assert_eq!(engine.calculate_specificity(Path::new("src/main.rs")), 2);
        assert_eq!(
            engine.calculate_specificity(Path::new("src/utils/helper.rs")),
            3
        );
    }

    #[test]
    fn test_precedence_rules() {
        let mut engine = SelectionEngine::new(vec![], vec![]);

        // Add less specific action first
        engine.exclude_file(PathBuf::from("src"));

        // Add more specific action later
        engine.include_file(PathBuf::from("src/main.rs"));

        // More specific should win
        assert!(!engine.is_selected(Path::new("src/lib.rs"))); // Excluded by src/
        assert!(engine.is_selected(Path::new("src/main.rs"))); // Included specifically
    }

    #[test]
    fn test_recent_wins_over_old() {
        let mut engine = SelectionEngine::new(vec![], vec![]);

        // First action
        engine.exclude_file(PathBuf::from("main.rs"));
        assert!(!engine.is_selected(Path::new("main.rs")));

        // More recent action with same specificity
        engine.include_file(PathBuf::from("main.rs"));
        assert!(engine.is_selected(Path::new("main.rs")));
    }
}
