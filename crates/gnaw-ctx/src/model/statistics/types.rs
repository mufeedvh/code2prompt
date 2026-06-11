//! Statistics view types and enums.
//!
//! This module contains the StatisticsView enum and related types
//! for managing different statistics views in the TUI.

/// Different views available in the Statistics tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsView {
    Overview,   // General statistics and summary
    TokenMap,   // Token distribution by directory/file
    Extensions, // Token distribution by file extension
}

impl StatisticsView {
    pub fn next(&self) -> Self {
        match self {
            StatisticsView::Overview => StatisticsView::TokenMap,
            StatisticsView::TokenMap => StatisticsView::Extensions,
            StatisticsView::Extensions => StatisticsView::Overview,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            StatisticsView::Overview => StatisticsView::Extensions,
            StatisticsView::TokenMap => StatisticsView::Overview,
            StatisticsView::Extensions => StatisticsView::TokenMap,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StatisticsView::Overview => "Overview",
            StatisticsView::TokenMap => "Token Map",
            StatisticsView::Extensions => "Extensions",
        }
    }
}
