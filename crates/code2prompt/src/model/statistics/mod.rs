//! Statistics state management for the TUI application.
//!
//! This module contains the statistics state and related functionality,
//! including different statistics views and their management.

pub mod types;

use crate::model::DisplayFileNode;
use crate::utils::format_number;
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
    /// Count selected files using session-based approach
    pub fn count_selected_files(
        session: &mut code2prompt_core::session::Code2PromptSession,
    ) -> usize {
        session.get_selected_files().unwrap_or_default().len()
    }

    /// Count total files in the tree nodes
    pub fn count_total_files(nodes: &[DisplayFileNode]) -> usize {
        fn rec(n: &DisplayFileNode) -> usize {
            if !n.is_directory {
                1
            } else {
                n.children.iter().map(rec).sum()
            }
        }
        nodes.iter().map(rec).sum()
    }

    /// Format number according to token format setting (moved from widget)
    pub fn format_number(
        num: usize,
        token_format: &code2prompt_core::tokenizer::TokenFormat,
    ) -> String {
        format_number(num, token_format)
    }

    /// Aggregate tokens by file extension (moved from widget - business logic belongs in Model)
    pub fn aggregate_by_extension(&self) -> Vec<(String, usize, usize)> {
        let mut extension_stats: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();

        for entry in &self.token_map_entries {
            if !entry.metadata.is_dir {
                let extension = entry
                    .name
                    .split('.')
                    .next_back()
                    .map(|ext| format!(".{}", ext))
                    .unwrap_or_else(|| "(no extension)".to_string());

                let (tokens, count) = extension_stats.entry(extension).or_insert((0, 0));
                *tokens += entry.tokens;
                *count += 1;
            }
        }

        // Convert to sorted vec (by tokens desc)
        let mut ext_vec: Vec<(String, usize, usize)> = extension_stats
            .into_iter()
            .map(|(ext, (tokens, count))| (ext, tokens, count))
            .collect();
        ext_vec.sort_by(|a, b| b.1.cmp(&a.1));
        ext_vec
    }
}
