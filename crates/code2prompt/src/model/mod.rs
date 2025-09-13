//! Data structures and application state management for the TUI.
//!
//! This module contains the core data structures that represent the application state,
//! including the main Model struct, tab definitions, message types for event handling,
//! and all state management submodules. It serves as the central state container
//! for the terminal user interface.

pub mod file_tree;
pub mod messages;
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

// Note: New message system available in messages module for future use
// pub use messages::{Message as NewMessage, StateChange, SystemEvent, UserAction};

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

/// Messages for updating the model (legacy enum for pattern matching)
#[derive(Debug, Clone)]
pub enum LegacyMessage {
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

    // Template focus and input handling
    SetTemplateFocus(TemplateFocus, FocusMode), // Set focus and mode
    SetTemplateFocusMode(FocusMode),            // Set focus mode only
    TemplateEditorInput(ratatui::crossterm::event::KeyEvent), // Direct textarea input
    TemplateVariableInput(ratatui::crossterm::event::KeyEvent), // Variable input
    TemplatePickerMove(i32),                    // Move picker cursor
}

// Type alias for backward compatibility
pub type Message = LegacyMessage;

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
    pub fn new(session: Code2PromptSession) -> Self {
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
        crate::view::format_settings_groups(&self.session.session)
    }

    // These methods will be used in the next phase of refactoring when Model::update() is implemented
    #[allow(dead_code)]
    /// Aggregate tokens by extension (moved from StatisticsByExtensionWidget)
    pub fn aggregate_tokens_by_extension(&self) -> Vec<(String, usize, usize)> {
        crate::view::aggregate_tokens_by_extension(&self.statistics.token_map_entries)
    }

    #[allow(dead_code)]
    /// Toggle directory selection and all its children (moved from FileSelectionWidget)
    pub fn toggle_directory_selection(&mut self, path: &str, selected: bool) {
        Self::update_node_selection_recursive(self.file_tree.get_file_tree_mut(), path, selected);
        Self::toggle_directory_children_selection(
            self.file_tree.get_file_tree_mut(),
            path,
            selected,
        );
    }

    #[allow(dead_code)]
    /// Update selection state of a specific node (moved from FileSelectionWidget)
    pub fn update_node_selection(&mut self, path: &str, selected: bool) {
        Self::update_node_selection_recursive(self.file_tree.get_file_tree_mut(), path, selected);
    }

    #[allow(dead_code)]
    /// Expand a directory in the file tree (moved from FileSelectionWidget)
    pub fn expand_directory(&mut self, path: &str) {
        Self::expand_directory_recursive(self.file_tree.get_file_tree_mut(), path);
    }

    #[allow(dead_code)]
    /// Collapse a directory in the file tree (moved from FileSelectionWidget)
    pub fn collapse_directory(&mut self, path: &str) {
        Self::collapse_directory_recursive(self.file_tree.get_file_tree_mut(), path);
    }

    #[allow(dead_code)]
    /// Adjust file tree scroll to keep cursor visible (moved from FileSelectionWidget)
    pub fn adjust_file_tree_scroll_for_cursor(&mut self) {
        let visible_count = self.file_tree.get_visible_nodes().len();
        if visible_count == 0 {
            return;
        }

        let viewport_height = 20; // This should match the actual content height in render
        let cursor_pos = self.file_tree.tree_cursor;
        let scroll_pos = self.file_tree.file_tree_scroll as usize;

        // If cursor is above viewport, scroll up
        if cursor_pos < scroll_pos {
            self.file_tree.file_tree_scroll = cursor_pos as u16;
        }
        // If cursor is below viewport, scroll down
        else if cursor_pos >= scroll_pos + viewport_height {
            self.file_tree.file_tree_scroll =
                (cursor_pos.saturating_sub(viewport_height - 1)) as u16;
        }

        // Ensure scroll doesn't go beyond bounds
        let max_scroll = visible_count.saturating_sub(viewport_height);
        if self.file_tree.file_tree_scroll as usize > max_scroll {
            self.file_tree.file_tree_scroll = max_scroll as u16;
        }
    }

    // Private helper methods (moved from FileSelectionWidget)
    #[allow(dead_code)]
    fn toggle_directory_children_selection(nodes: &mut [FileNode], dir_path: &str, selected: bool) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == dir_path && node.is_directory {
                Self::select_all_children(&mut node.children, selected);
                return;
            }
            Self::toggle_directory_children_selection(&mut node.children, dir_path, selected);
        }
    }

    #[allow(dead_code)]
    fn select_all_children(nodes: &mut [FileNode], selected: bool) {
        for node in nodes.iter_mut() {
            node.is_selected = selected;
            if node.is_directory {
                Self::select_all_children(&mut node.children, selected);
            }
        }
    }

    #[allow(dead_code)]
    fn update_node_selection_recursive(nodes: &mut [FileNode], path: &str, selected: bool) -> bool {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == path {
                node.is_selected = selected;
                return true;
            }
            if Self::update_node_selection_recursive(&mut node.children, path, selected) {
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    fn expand_directory_recursive(nodes: &mut [FileNode], path: &str) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == path && node.is_directory {
                node.is_expanded = true;
                // Load children if not already loaded
                if node.children.is_empty() {
                    if let Ok(entries) = std::fs::read_dir(&node.path) {
                        for entry in entries.flatten() {
                            let child_path = entry.path();
                            let mut child_node = FileNode::new(child_path, node.level + 1);
                            child_node.is_selected = false;
                            node.children.push(child_node);
                        }
                        // Sort children
                        node.children
                            .sort_by(|a, b| match (a.is_directory, b.is_directory) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.name.cmp(&b.name),
                            });
                    }
                }
                return;
            }
            Self::expand_directory_recursive(&mut node.children, path);
        }
    }

    #[allow(dead_code)]
    fn collapse_directory_recursive(nodes: &mut [FileNode], target_path: &str) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == target_path && node.is_directory {
                node.is_expanded = false;
                return;
            }
            Self::collapse_directory_recursive(&mut node.children, target_path);
        }
    }
}
