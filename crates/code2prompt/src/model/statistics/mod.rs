//! Statistics state management for the TUI application.
//!
//! This module contains the statistics state and related functionality,
//! including different statistics views and their management.

pub mod types;

use crate::model::FileNode;
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
