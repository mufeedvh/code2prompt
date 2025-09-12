//! Data structures and application state management for the TUI.
//!
//! This module contains the core data structures that represent the application state,
//! including the main Model struct, tab definitions, message types for event handling,
//! and all state management submodules. It serves as the central state container
//! for the terminal user interface.

pub mod file_tree;
pub mod prompt_output;
pub mod session;
pub mod settings;
pub mod statistics;
pub mod template;

pub use file_tree::*;
pub use prompt_output::*;
pub use session::*;
pub use settings::*;
pub use statistics::*;
pub use template::*;

use code2prompt_core::session::Code2PromptSession;

/// The five main tabs of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FileTree,     // Tab 1: File selection with search
    Settings,     // Tab 2: Configuration options
    Statistics,   // Tab 3: Analysis statistics and metrics
    Template,     // Tab 4: Template editor
    PromptOutput, // Tab 5: Generated prompt and copy
}

/// Messages for updating the model
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    SwitchTab(Tab),
    Quit,

    // File tree
    UpdateSearchQuery(String),
    ToggleFileSelection(usize),
    ExpandDirectory(usize),
    CollapseDirectory(usize),
    MoveTreeCursor(i32),
    RefreshFileTree,

    // Search mode
    EnterSearchMode,
    ExitSearchMode,

    // Settings
    MoveSettingsCursor(i32),
    ToggleSetting(usize),
    CycleSetting(usize),

    // Analysis
    RunAnalysis,
    AnalysisComplete(AnalysisResults), // Complete analysis results
    AnalysisError(String),

    // Prompt output
    CopyToClipboard,
    SaveToFile(String),
    ScrollOutput(i16),   // Scroll delta (positive = down, negative = up)
    ScrollFileTree(i16), // Scroll delta for file tree

    // Statistics
    CycleStatisticsView(i8), // +1 = next view, -1 = previous view
    ScrollStatistics(i16),   // Scroll delta for statistics

    // Template - Redux/Elm style messages
    SaveTemplate(String), // Save template with name
    ReloadTemplate,
    LoadTemplate,     // Load selected template from picker
    RefreshTemplates, // Refresh template list
}

/// Represents the overall state of the TUI application.
#[derive(Debug, Clone)]
pub struct Model {
    pub session: SessionState,
    pub current_tab: Tab,
    pub should_quit: bool,
    pub file_tree: FileTreeState,
    pub settings: SettingsState,
    pub statistics: StatisticsState,
    pub template: TemplateState,
    pub prompt_output: PromptOutputState,
    pub status_message: String,
}

impl Default for Model {
    fn default() -> Self {
        let session_state = SessionState::default();

        Model {
            session: session_state,
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree: FileTreeState::default(),
            settings: SettingsState::default(),
            statistics: StatisticsState::default(),
            template: TemplateState::default(),
            prompt_output: PromptOutputState::default(),
            status_message: String::new(),
        }
    }
}

impl Model {
    pub fn new_with_cli_args(session: Code2PromptSession) -> Self {
        Model {
            session: SessionState::new(session),
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree: FileTreeState::default(),
            settings: SettingsState::default(),
            statistics: StatisticsState::default(),
            template: TemplateState::default(),
            prompt_output: PromptOutputState::default(),
            status_message: String::new(),
        }
    }

    /// Get grouped settings for display
    pub fn get_settings_groups(&self) -> Vec<SettingsGroup> {
        self.settings.get_settings_groups(&self.session.session)
    }
}
