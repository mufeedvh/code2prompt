//! Extensions statistics state and functionality.
//!
//! This module contains state and logic specific to the extensions
//! statistics view in the TUI.

/// Extensions state for the statistics extensions view
#[derive(Debug, Clone)]
pub struct ExtensionsState {
    // Add specific state if needed in the future
    // For now, the extensions view uses shared statistics state
}

impl Default for ExtensionsState {
    fn default() -> Self {
        ExtensionsState {}
    }
}

impl ExtensionsState {
    /// Create a new extensions state
    pub fn new() -> Self {
        Self::default()
    }
}
