//! Statistics state management for the TUI application.
//!
//! This module contains the statistics state and related functionality,
//! including different statistics views and their management.

pub mod types;

pub use types::*;

/// Statistics state containing all statistics-related data
#[derive(Debug, Clone)]
pub struct StatisticsState {
    pub view: StatisticsView,
    pub scroll: u16,
    pub token_map_entries: Vec<crate::token_map::TokenMapEntry>,
}

impl Default for StatisticsState {
    fn default() -> Self {
        StatisticsState {
            view: StatisticsView::Overview,
            scroll: 0,
            token_map_entries: Vec::new(),
        }
    }
}
