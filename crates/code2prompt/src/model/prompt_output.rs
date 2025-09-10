//! Prompt output state management for the TUI application.
//!
//! This module contains the prompt output state and related functionality
//! for managing generated prompts and analysis results in the TUI.

/// Prompt output state containing all prompt output related data
#[derive(Debug, Clone)]
pub struct PromptOutputState {
    pub generated_prompt: Option<String>,
    pub token_count: Option<usize>,
    pub file_count: usize,
    pub analysis_in_progress: bool,
    pub analysis_error: Option<String>,
    pub output_scroll: u16,
}

/// Results from code2prompt analysis
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    pub file_count: usize,
    pub token_count: Option<usize>,
    pub generated_prompt: String,
    pub token_map_entries: Vec<crate::token_map::TokenMapEntry>,
}

impl Default for PromptOutputState {
    fn default() -> Self {
        PromptOutputState {
            generated_prompt: None,
            token_count: None,
            file_count: 0,
            analysis_in_progress: false,
            analysis_error: None,
            output_scroll: 0,
        }
    }
}
