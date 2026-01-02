//! Prompt output state management for the TUI application.
//!
//! This module contains the prompt output state and related functionality
//! for managing generated prompts and analysis results in the TUI.
use code2prompt_core::analysis::TokenMapEntry;
use code2prompt_core::session::RenderedPrompt;

/// Prompt output state containing all prompt output related data
///
/// This struct stores the complete RenderedPrompt from the Core, preserving
/// all context (files, directory_name, model_info, etc.) as a Single Source of Truth.
#[derive(Debug, Clone)]
pub struct PromptOutputState {
    /// The complete rendered prompt result from the Core
    pub result: Option<RenderedPrompt>,
    pub analysis_in_progress: bool,
    pub analysis_error: Option<String>,
    pub output_scroll: u16,
}

impl Default for PromptOutputState {
    fn default() -> Self {
        Self {
            result: None,
            analysis_in_progress: false,
            analysis_error: None,
            output_scroll: 0,
        }
    }
}

/// Results from code2prompt analysis
///
/// Contains the complete RenderedPrompt from generation plus additional
/// analysis data (token map). No data duplication.
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    /// The complete rendered prompt result
    pub rendered: RenderedPrompt,
    /// Token map entries for statistics
    pub token_map_entries: Vec<TokenMapEntry>,
}
