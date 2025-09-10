//! Token map statistics state and functionality.
//!
//! This module contains state and logic specific to the token map
//! statistics view in the TUI.

/// Token map state for the statistics token map view
#[derive(Debug, Clone)]
pub struct TokenMapState {
    // Add specific state if needed in the future
    // For now, the token map view uses shared statistics state
}

impl Default for TokenMapState {
    fn default() -> Self {
        TokenMapState {}
    }
}

impl TokenMapState {
    /// Create a new token map state
    pub fn new() -> Self {
        Self::default()
    }
}
