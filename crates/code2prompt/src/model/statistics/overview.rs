//! Overview statistics state and functionality.
//!
//! This module contains state and logic specific to the overview
//! statistics view in the TUI.

use crate::model::Model;

/// Overview state for the statistics overview view
#[derive(Debug, Clone)]
pub struct OverviewState {
    pub scroll_offset: u16,
    pub selected_metric: usize,
}

impl Default for OverviewState {
    fn default() -> Self {
        OverviewState {
            scroll_offset: 0,
            selected_metric: 0,
        }
    }
}

impl OverviewState {
    /// Create a new overview state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create overview state from model
    pub fn from_model(_model: &Model) -> Self {
        Self::default()
    }

    /// Get available metrics for navigation
    pub fn get_metrics(&self) -> Vec<&'static str> {
        vec![
            "Total Files",
            "Total Tokens",
            "Average Tokens per File",
            "File Types",
            "Largest Files",
        ]
    }

    /// Move cursor to next metric
    pub fn next_metric(&mut self) {
        let metrics_count = self.get_metrics().len();
        if metrics_count > 0 {
            self.selected_metric = (self.selected_metric + 1) % metrics_count;
        }
    }

    /// Move cursor to previous metric
    pub fn prev_metric(&mut self) {
        let metrics_count = self.get_metrics().len();
        if metrics_count > 0 {
            self.selected_metric = if self.selected_metric == 0 {
                metrics_count - 1
            } else {
                self.selected_metric - 1
            };
        }
    }
}
