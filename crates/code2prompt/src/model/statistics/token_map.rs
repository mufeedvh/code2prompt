//! Token map statistics state and functionality.
//!
//! This module contains state and logic specific to the token map
//! statistics view in the TUI.

use crate::model::Model;

/// Token map state for the statistics token map view
#[derive(Debug, Clone)]
pub struct TokenMapState {
    pub scroll_offset: u16,
    pub selected_file: usize,
    pub sort_by_tokens: bool,
}

impl Default for TokenMapState {
    fn default() -> Self {
        TokenMapState {
            scroll_offset: 0,
            selected_file: 0,
            sort_by_tokens: true,
        }
    }
}

impl TokenMapState {
    /// Create a new token map state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create token map state from model
    pub fn from_model(_model: &Model) -> Self {
        Self::default()
    }

    /// Move cursor to next file
    pub fn next_file(&mut self, total_files: usize) {
        if total_files > 0 {
            self.selected_file = (self.selected_file + 1).min(total_files - 1);
        }
    }

    /// Move cursor to previous file
    pub fn prev_file(&mut self) {
        if self.selected_file > 0 {
            self.selected_file -= 1;
        }
    }

    /// Toggle sort order between tokens and filename
    pub fn toggle_sort(&mut self) {
        self.sort_by_tokens = !self.sort_by_tokens;
    }

    /// Get sort description
    pub fn get_sort_description(&self) -> &'static str {
        if self.sort_by_tokens {
            "Sorted by tokens (descending)"
        } else {
            "Sorted by filename"
        }
    }
}
