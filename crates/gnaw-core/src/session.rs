//! Interactive selection state for the TUI.
//!
//! `SelectionState` holds the user's working configuration and their live file
//! selection (`SelectionEngine`). It is the stateful object the TUI mutates as
//! the user toggles files and edits settings; when the user runs an analysis,
//! the frontend snapshots the selection and drives the pipeline (`build_spec` +
//! `run`) — this type no longer loads codebases or renders prompts itself. The
//! old eager load/render path (and its separate, drift-prone token estimator)
//! lived here and was deleted with the pipeline migration.

use anyhow::Result;
use std::path::PathBuf;

use crate::configuration::GnawConfig;
use crate::selection::SelectionEngine;

/// User-facing session state: working config plus the live file selection.
/// Mutated by the TUI; read (via `get_selected_files`) when building a pipeline
/// run. Holds no loaded codebase or rendered output — those are the pipeline's.
#[derive(Debug, Clone)]
pub struct SelectionState {
    pub config: GnawConfig,
    pub selection_engine: SelectionEngine,
}

impl SelectionState {
    /// Creates a new session with SelectionEngine for pattern-based and user-driven file selection
    pub fn new(config: GnawConfig) -> Self {
        let selection_engine = SelectionEngine::new(
            config.include_patterns.clone(),
            config.exclude_patterns.clone(),
            config.deselected,
        );

        Self {
            selection_engine,
            config,
        }
    }

    /// Add pattern and recreate SelectionEngine
    pub fn add_include_pattern(&mut self, pattern: String) -> &mut Self {
        self.config.include_patterns.push(pattern);
        // Recreate SelectionEngine with new patterns
        self.selection_engine = SelectionEngine::new(
            self.config.include_patterns.clone(),
            self.config.exclude_patterns.clone(),
            self.config.deselected,
        );
        self
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) -> &mut Self {
        self.config.exclude_patterns.push(pattern);
        // Recreate SelectionEngine with new patterns
        self.selection_engine = SelectionEngine::new(
            self.config.include_patterns.clone(),
            self.config.exclude_patterns.clone(),
            self.config.deselected,
        );
        self
    }

    /// User interaction: include a file (delegates to SelectionEngine)
    pub fn select_file(&mut self, path: PathBuf) -> &mut Self {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(&self.config.path)
                .unwrap_or(&path)
                .to_path_buf()
        } else {
            path
        };

        self.selection_engine.include_file(relative_path);
        self
    }

    /// User interaction: exclude a file (delegates to SelectionEngine)
    pub fn deselect_file(&mut self, path: PathBuf) -> &mut Self {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(&self.config.path)
                .unwrap_or(&path)
                .to_path_buf()
        } else {
            path
        };

        self.selection_engine.exclude_file(relative_path);
        self
    }

    /// User interaction: toggle file selection (delegates to SelectionEngine)
    pub fn toggle_file_selection(&mut self, path: PathBuf) -> &mut Self {
        let relative_path = match path.strip_prefix(&self.config.path) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => path,
        };
        self.selection_engine.toggle_file(relative_path);
        self
    }

    /// Check if a file is selected (delegates to SelectionEngine)
    pub fn is_file_selected(&mut self, path: &std::path::Path) -> bool {
        // Always normalize to a root-relative path. When `config.path` is itself
        // relative (e.g. "." from `gnaw .`), walker paths look like
        // "./crates/foo.rs" — still carrying a CurDir component — so the old
        // `is_absolute()` guard left them un-stripped, and they failed to match
        // selection actions recorded as "crates/foo.rs".
        let relative_path = path.strip_prefix(&self.config.path).unwrap_or(path);
        self.selection_engine.is_selected(relative_path)
    }

    /// Get all currently selected files (delegates to SelectionEngine)
    pub fn get_selected_files(&mut self) -> Result<Vec<PathBuf>> {
        Ok(self
            .selection_engine
            .get_selected_files(&self.config.path)?)
    }

    /// Clear all user actions (reset to pattern-only behavior)
    pub fn clear_user_actions(&mut self) -> &mut Self {
        self.selection_engine.clear_user_actions();
        self
    }

    /// Check if there are any user actions beyond base patterns
    pub fn has_user_actions(&self) -> bool {
        self.selection_engine.has_user_actions()
    }

    /// Set deselected by default and update selection engine
    pub fn set_deselected(&mut self, value: bool) -> &mut Self {
        self.config.deselected = value;
        self.selection_engine.set_deselected_by_default(value);
        self
    }
}
