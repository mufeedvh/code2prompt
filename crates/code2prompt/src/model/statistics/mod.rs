//! Statistics state management for the TUI application.
//!
//! This module contains the statistics state and related functionality,
//! including different statistics views and their management.

pub mod types;

use crate::model::{FileNode, Message};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
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

impl StatisticsState {
    /// Handle key events for statistics (moved from widget)
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => Some(Message::RunAnalysis),
            KeyCode::Left => Some(Message::CycleStatisticsView(-1)), // Previous view
            KeyCode::Right => Some(Message::CycleStatisticsView(1)), // Next view
            KeyCode::Up => Some(Message::ScrollStatistics(-1)),
            KeyCode::Down => Some(Message::ScrollStatistics(1)),
            KeyCode::PageUp => Some(Message::ScrollStatistics(-5)),
            KeyCode::PageDown => Some(Message::ScrollStatistics(5)),
            KeyCode::Home => Some(Message::ScrollStatistics(-9999)),
            KeyCode::End => Some(Message::ScrollStatistics(9999)),
            _ => None,
        }
    }

    /// Count selected files in the tree (moved from widget)
    pub fn count_selected_files(nodes: &[FileNode]) -> usize {
        let mut count = 0;
        for node in nodes {
            if node.is_selected && !node.is_directory {
                count += 1;
            }
            count += Self::count_selected_files(&node.children);
        }
        count
    }

    /// Count total files in the tree (moved from widget)
    pub fn count_total_files(nodes: &[FileNode]) -> usize {
        let mut count = 0;
        for node in nodes {
            if !node.is_directory {
                count += 1;
            }
            count += Self::count_total_files(&node.children);
        }
        count
    }

    /// Format number according to token format setting (moved from widget)
    pub fn format_number(
        num: usize,
        token_format: &code2prompt_core::tokenizer::TokenFormat,
    ) -> String {
        use code2prompt_core::tokenizer::TokenFormat;
        use num_format::{SystemLocale, ToFormattedString};

        match token_format {
            TokenFormat::Format => SystemLocale::default()
                .map(|locale| num.to_formatted_string(&locale))
                .unwrap_or_else(|_| num.to_string()),
            TokenFormat::Raw => num.to_string(),
        }
    }
}
