//! Statistics state management for the TUI application.
//!
//! This module contains the statistics state and related functionality,
//! including different statistics views and their management.

pub mod types;

use code2prompt_core::analysis::{CodebaseAnalysis, TokenMapEntry};

use crate::model::DisplayFileNode;
use crate::utils::format_number;
pub use types::*;

/// Statistics state containing all statistics-related data
#[derive(Debug, Clone)]
pub struct StatisticsState {
    pub view: StatisticsView,
    pub scroll: u16,
    pub token_map_entries: Vec<TokenMapEntry>,
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

    /// Aggregate tokens by file extension using a CodebaseAnalysis
    ///
    /// This method delegates to the analysis facade's by_extension() method.
    /// The caller can provide either a raw or contextual analysis depending on their needs.
    ///
    /// # Arguments
    ///
    /// * `analysis` - A CodebaseAnalysis instance (raw or contextual)
    ///
    /// # Returns
    ///
    /// * `Vec<(extension, tokens, file_count)>` - Aggregated statistics
    pub fn aggregate_by_extension(analysis: &CodebaseAnalysis) -> Vec<(String, usize, usize)> {
        let ext_stats = analysis.by_extension();

        // Convert to the format expected by the widget (extension, tokens, count)
        ext_stats
            .into_iter()
            .map(|stat| (stat.extension, stat.tokens, stat.file_count))
            .collect()
    }
}
