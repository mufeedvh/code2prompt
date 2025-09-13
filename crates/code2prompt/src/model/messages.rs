//! Message types for the Model-View-Update architecture.
//!
//! This module organizes messages into three clear categories:
//! - UserAction: Actions initiated by user input
//! - SystemEvent: Events from the system or async operations
//! - StateChange: Direct state modifications
//!
//! This separation makes the data flow clearer and follows Elm/Redux patterns.

use crate::model::{AnalysisResults, FocusMode, Tab, TemplateFocus};
use ratatui::crossterm::event::KeyEvent;

/// Actions initiated by user input (keyboard, mouse, etc.)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when Model::update() is implemented
pub enum UserAction {
    // Navigation
    SwitchTab(Tab),
    Quit,

    // File tree interactions
    ToggleFileSelection(usize),
    ExpandDirectory(usize),
    CollapseDirectory(usize),
    EnterSearchMode,
    ExitSearchMode,

    // Settings interactions
    ToggleSetting(usize),
    CycleSetting(usize),

    // Analysis
    RunAnalysis,

    // Prompt output
    CopyToClipboard,
    SaveToFile(String),

    // Statistics
    CycleStatisticsView(i8), // +1 = next view, -1 = previous view

    // Template interactions
    SaveTemplate(String),
    ReloadTemplate,
    LoadTemplate,
    RefreshTemplates,
    SetTemplateFocus(TemplateFocus, FocusMode),
    SetTemplateFocusMode(FocusMode),
}

/// Events from the system or async operations
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when Model::update() is implemented
pub enum SystemEvent {
    // Analysis results
    AnalysisComplete(AnalysisResults),
    AnalysisError(String),

    // File system events
    FileTreeRefreshed,
    FileTreeError(String),

    // Template events
    TemplateLoaded(String),
    TemplateSaved(String),
    TemplateError(String),
}

/// Direct state modifications (usually triggered by UserActions)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when Model::update() is implemented
pub enum StateChange {
    // Navigation state
    UpdateCurrentTab(Tab),
    SetShouldQuit(bool),

    // File tree state
    UpdateSearchQuery(String),
    MoveTreeCursor(i32),
    ScrollFileTree(i16),
    RefreshFileTree,

    // Settings state
    MoveSettingsCursor(i32),

    // Output state
    ScrollOutput(i16),

    // Statistics state
    ScrollStatistics(i16),

    // Template state
    TemplateEditorInput(KeyEvent),
    TemplateVariableInput(KeyEvent),
    TemplatePickerMove(i32),

    // Status
    UpdateStatusMessage(String),
}

/// Unified message type that encompasses all message categories
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when Model::update() is implemented
pub enum Message {
    UserAction(UserAction),
    SystemEvent(SystemEvent),
    StateChange(StateChange),
}

// Convenience constructors for easier usage
impl From<UserAction> for Message {
    fn from(action: UserAction) -> Self {
        Message::UserAction(action)
    }
}

impl From<SystemEvent> for Message {
    fn from(event: SystemEvent) -> Self {
        Message::SystemEvent(event)
    }
}

impl From<StateChange> for Message {
    fn from(change: StateChange) -> Self {
        Message::StateChange(change)
    }
}

// Legacy compatibility - these will be removed after migration
// Note: Constructor functions will be added in future phases when migrating to pure Elm/Redux pattern
// For now, we use the legacy enum directly to avoid unused code warnings
