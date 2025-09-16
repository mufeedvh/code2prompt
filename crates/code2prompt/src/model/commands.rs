//! Command system for handling side effects in the Model-View-Update architecture.
//!
//! This module implements the Cmd pattern from Elm/Redux, allowing the Model::update()
//! function to remain pure while still triggering side effects like async operations,
//! file I/O, and clipboard operations.

use code2prompt_core::session::Code2PromptSession;
use std::collections::HashMap;

/// Commands represent side effects that should be executed after model updates.
/// This allows Model::update() to remain pure while still triggering necessary
/// side effects like async operations, file I/O, etc.
#[derive(Debug, Clone)]
pub enum Cmd {
    /// No command - pure state update only
    None,

    /// Run analysis in background
    RunAnalysis {
        session: Box<Code2PromptSession>,
        template_content: String,
        user_variables: HashMap<String, String>,
    },

    /// Copy text to clipboard
    CopyToClipboard(String),

    /// Save text to file
    SaveToFile { filename: String, content: String },

    /// Save template to custom directory
    SaveTemplate { filename: String, content: String },

    /// Refresh file tree from session
    RefreshFileTree,
}
