//! Extensions statistics state and functionality.
//!
//! This module contains state and logic specific to the extensions
//! statistics view in the TUI.

use crate::model::Model;

/// Extensions state for the statistics extensions view
#[derive(Debug, Clone)]
pub struct ExtensionsState {
    pub scroll_offset: u16,
    pub selected_extension: usize,
    pub sort_by_count: bool,
    pub show_details: bool,
}

impl Default for ExtensionsState {
    fn default() -> Self {
        ExtensionsState {
            scroll_offset: 0,
            selected_extension: 0,
            sort_by_count: true,
            show_details: false,
        }
    }
}

impl ExtensionsState {
    /// Create a new extensions state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create extensions state from model
    pub fn from_model(_model: &Model) -> Self {
        Self::default()
    }

    /// Move cursor to next extension
    pub fn next_extension(&mut self, total_extensions: usize) {
        if total_extensions > 0 {
            self.selected_extension = (self.selected_extension + 1).min(total_extensions - 1);
        }
    }

    /// Move cursor to previous extension
    pub fn prev_extension(&mut self) {
        if self.selected_extension > 0 {
            self.selected_extension -= 1;
        }
    }

    /// Toggle sort order between count and extension name
    pub fn toggle_sort(&mut self) {
        self.sort_by_count = !self.sort_by_count;
    }

    /// Toggle details view
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    /// Get sort description
    pub fn get_sort_description(&self) -> &'static str {
        if self.sort_by_count {
            "Sorted by file count (descending)"
        } else {
            "Sorted by extension name"
        }
    }

    /// Get view description
    pub fn get_view_description(&self) -> &'static str {
        if self.show_details {
            "Detailed view with file lists"
        } else {
            "Summary view"
        }
    }
}
