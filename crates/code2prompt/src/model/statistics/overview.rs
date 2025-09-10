//! Overview statistics state and functionality.
//!
//! This module contains state and logic specific to the overview
//! statistics view in the TUI.

/// Overview state for the statistics overview view
#[derive(Debug, Clone)]
pub struct OverviewState {
    // Add specific state if needed in the future
    // For now, the overview view uses shared statistics state
}

impl Default for OverviewState {
    fn default() -> Self {
        OverviewState {}
    }
}

impl OverviewState {
    /// Create a new overview state
    pub fn new() -> Self {
        Self::default()
    }
}
