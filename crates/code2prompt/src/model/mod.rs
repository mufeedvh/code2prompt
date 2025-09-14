//! Data structures and application state management for the TUI.
//!
//! This module contains the core data structures that represent the application state,
//! including the main Model struct, tab definitions, message types for event handling,
//! and all state management submodules. It serves as the central state container
//! for the terminal user interface.

pub mod commands;
pub mod file_tree;
pub mod prompt_output;
pub mod session;
pub mod settings;
pub mod statistics;
pub mod template;

pub use commands::*;
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
    FileTree,
    Settings,
    Statistics,
    Template,
    PromptOutput,
}

/// Messages for updating the model
#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    Quit,

    UpdateSearchQuery(String),
    ToggleFileSelection(usize),
    ExpandDirectory(usize),
    CollapseDirectory(usize),
    MoveTreeCursor(i32),
    RefreshFileTree,

    EnterSearchMode,
    ExitSearchMode,

    MoveSettingsCursor(i32),
    ToggleSetting(usize),
    CycleSetting(usize),

    RunAnalysis,
    AnalysisComplete(AnalysisResults),
    AnalysisError(String),

    CopyToClipboard,
    SaveToFile(String),
    ScrollOutput(i16),

    CycleStatisticsView(i8),
    ScrollStatistics(i16),

    SaveTemplate(String),
    ReloadTemplate,
    LoadTemplate,
    RefreshTemplates,

    SetTemplateFocus(TemplateFocus, FocusMode),
    SetTemplateFocusMode(FocusMode),
    TemplateEditorInput(ratatui::crossterm::event::KeyEvent),
    TemplatePickerMove(i32),

    VariableStartEditing(String),
    VariableInputChar(char),
    VariableInputBackspace,
    VariableInputEnter,
    VariableInputCancel,
    VariableNavigateUp,
    VariableNavigateDown,
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

    pub fn update(&self, message: Message) -> (Self, Cmd) {
        let mut new_model = self.clone();

        match message {
            Message::Quit => {
                new_model.should_quit = true;
                new_model.status_message = "Goodbye!".to_string();
                (new_model, Cmd::None)
            }

            Message::SwitchTab(tab) => {
                new_model.current_tab = tab;
                new_model.status_message = format!("Switched to {:?} tab", tab);
                (new_model, Cmd::None)
            }

            Message::RefreshFileTree => {
                new_model.status_message = "Refreshing file tree...".to_string();
                (new_model, Cmd::RefreshFileTree)
            }

            Message::UpdateSearchQuery(query) => {
                new_model.file_tree.search_query = query;
                new_model.file_tree.tree_cursor = 0; // Reset cursor when search changes
                (new_model, Cmd::None)
            }

            Message::EnterSearchMode => {
                new_model.status_message = "Search mode - Type to search, Esc to exit".to_string();
                (new_model, Cmd::None)
            }

            Message::ExitSearchMode => {
                new_model.status_message = "Exited search mode".to_string();
                (new_model, Cmd::None)
            }

            Message::MoveTreeCursor(delta) => {
                let visible_count = new_model.file_tree.get_visible_nodes().len();
                if visible_count > 0 {
                    let new_cursor = if delta > 0 {
                        (new_model.file_tree.tree_cursor + delta as usize).min(visible_count - 1)
                    } else {
                        new_model
                            .file_tree
                            .tree_cursor
                            .saturating_sub((-delta) as usize)
                    };
                    new_model.file_tree.tree_cursor = new_cursor;

                    let viewport_height = 20;
                    let cursor_pos = new_model.file_tree.tree_cursor as u16;
                    let current_scroll = new_model.file_tree.file_tree_scroll;

                    if cursor_pos < current_scroll {
                        // Cursor is above visible area, scroll up
                        new_model.file_tree.file_tree_scroll = cursor_pos;
                    } else if cursor_pos >= current_scroll + viewport_height {
                        // Cursor is below visible area, scroll down
                        new_model.file_tree.file_tree_scroll =
                            cursor_pos.saturating_sub(viewport_height - 1);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::MoveSettingsCursor(delta) => {
                let settings_count = new_model
                    .settings
                    .get_settings_items(&new_model.session.session)
                    .len();
                if settings_count > 0 {
                    let new_cursor = if delta > 0 {
                        (new_model.settings.settings_cursor + delta as usize)
                            .min(settings_count - 1)
                    } else {
                        new_model
                            .settings
                            .settings_cursor
                            .saturating_sub((-delta) as usize)
                    };
                    new_model.settings.settings_cursor = new_cursor;
                }
                (new_model, Cmd::None)
            }

            Message::ToggleFileSelection(index) => {
                let visible_nodes = new_model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    let path = node.path.to_string_lossy().to_string();
                    let name = node.name.clone();
                    let is_directory = node.is_directory;
                    let current = node.is_selected;

                    // Update session selection
                    if !current {
                        new_model.session.session.include_file(node.path.clone());
                    } else {
                        new_model.session.session.exclude_file(node.path.clone());
                    }

                    new_model
                        .file_tree
                        .update_node_selection(&path, !current, is_directory);

                    let action = if current { "Deselected" } else { "Selected" };
                    let extra = if is_directory { " (and contents)" } else { "" };
                    new_model.status_message = format!("{} {}{}", action, name, extra);
                }
                (new_model, Cmd::None)
            }

            Message::ExpandDirectory(index) => {
                let visible_nodes = new_model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    if node.is_directory {
                        let path = node.path.to_string_lossy().to_string();
                        let name = node.name.clone();
                        new_model.file_tree.expand_directory(&path);
                        new_model.status_message = format!("Expanded {}", name);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::CollapseDirectory(index) => {
                let visible_nodes = new_model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    if node.is_directory {
                        let path = node.path.to_string_lossy().to_string();
                        let name = node.name.clone();
                        new_model.file_tree.collapse_directory(&path);
                        new_model.status_message = format!("Collapsed {}", name);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::ToggleSetting(index) => {
                new_model.settings.update_setting(
                    &mut new_model.session.session,
                    index,
                    SettingAction::Toggle,
                );
                let settings = new_model
                    .settings
                    .get_settings_items(&new_model.session.session);
                if let Some(setting) = settings.get(index) {
                    new_model.status_message = format!("Toggled {}", setting.name);
                }
                (new_model, Cmd::None)
            }

            Message::CycleSetting(index) => {
                new_model.settings.update_setting(
                    &mut new_model.session.session,
                    index,
                    SettingAction::Cycle,
                );
                let settings = new_model
                    .settings
                    .get_settings_items(&new_model.session.session);
                if let Some(setting) = settings.get(index) {
                    new_model.status_message = format!("Cycled {}", setting.name);
                }
                (new_model, Cmd::None)
            }

            Message::RunAnalysis => {
                if !new_model.prompt_output.analysis_in_progress {
                    new_model.prompt_output.analysis_in_progress = true;
                    new_model.prompt_output.analysis_error = None;
                    new_model.status_message = "Running analysis...".to_string();
                    new_model.current_tab = Tab::PromptOutput; // Switch to output tab

                    let cmd = Cmd::RunAnalysis {
                        session: Box::new(new_model.session.session.clone()),
                        template_content: new_model.template.get_template_content().to_string(),
                    };
                    (new_model, cmd)
                } else {
                    new_model.status_message = "Analysis already in progress...".to_string();
                    (new_model, Cmd::None)
                }
            }

            Message::AnalysisComplete(results) => {
                new_model.prompt_output.analysis_in_progress = false;
                new_model.prompt_output.generated_prompt = Some(results.generated_prompt);
                new_model.prompt_output.token_count = results.token_count;
                new_model.prompt_output.file_count = results.file_count;
                new_model.statistics.token_map_entries = results.token_map_entries;
                let tokens = results.token_count.unwrap_or(0);
                new_model.status_message = format!(
                    "Analysis complete! {} tokens, {} files",
                    tokens, results.file_count
                );
                (new_model, Cmd::None)
            }

            Message::AnalysisError(error) => {
                new_model.prompt_output.analysis_in_progress = false;
                new_model.prompt_output.analysis_error = Some(error.clone());
                new_model.status_message = format!("Analysis failed: {}", error);
                (new_model, Cmd::None)
            }

            Message::CopyToClipboard => {
                if let Some(prompt) = &new_model.prompt_output.generated_prompt {
                    let cmd = Cmd::CopyToClipboard(prompt.clone());
                    (new_model, cmd)
                } else {
                    new_model.status_message = "No prompt to copy".to_string();
                    (new_model, Cmd::None)
                }
            }

            Message::SaveToFile(filename) => {
                if let Some(prompt) = &new_model.prompt_output.generated_prompt {
                    let cmd = Cmd::SaveToFile {
                        filename,
                        content: prompt.clone(),
                    };
                    (new_model, cmd)
                } else {
                    new_model.status_message = "No prompt to save".to_string();
                    (new_model, Cmd::None)
                }
            }

            Message::ScrollOutput(delta) => {
                if let Some(prompt) = &new_model.prompt_output.generated_prompt {
                    let lines = prompt.lines().count() as u16;
                    let viewport_height = 20; // Approximate viewport height
                    let max_scroll = lines.saturating_sub(viewport_height);

                    let new_scroll = if delta < 0 {
                        new_model
                            .prompt_output
                            .output_scroll
                            .saturating_sub((-delta) as u16)
                    } else {
                        new_model
                            .prompt_output
                            .output_scroll
                            .saturating_add(delta as u16)
                    };

                    new_model.prompt_output.output_scroll = new_scroll.min(max_scroll);
                }
                (new_model, Cmd::None)
            }

            Message::CycleStatisticsView(direction) => {
                new_model.statistics.view = if direction > 0 {
                    new_model.statistics.view.next()
                } else {
                    new_model.statistics.view.prev()
                };
                new_model.statistics.scroll = 0;
                new_model.status_message =
                    format!("Switched to {} view", new_model.statistics.view.as_str());
                (new_model, Cmd::None)
            }

            Message::ScrollStatistics(delta) => {
                let new_scroll = if delta < 0 {
                    new_model.statistics.scroll.saturating_sub((-delta) as u16)
                } else {
                    new_model.statistics.scroll.saturating_add(delta as u16)
                };
                new_model.statistics.scroll = new_scroll;
                (new_model, Cmd::None)
            }

            Message::SaveTemplate(filename) => {
                let content = new_model.template.get_template_content().to_string();
                let cmd = Cmd::SaveTemplate {
                    filename: filename.clone(),
                    content,
                };
                new_model.status_message = "Saving template...".to_string();
                (new_model, cmd)
            }

            Message::ReloadTemplate => {
                new_model.template.editor = crate::model::template::EditorState::default();
                new_model.template.sync_variables_with_template();
                new_model.status_message = "Reloaded template".to_string();
                (new_model, Cmd::None)
            }

            Message::LoadTemplate => {
                let result = new_model.template.load_selected_template();
                match result {
                    Ok(template_name) => {
                        new_model.template.sync_variables_with_template();
                        new_model.status_message = format!("Loaded template: {}", template_name);
                    }
                    Err(e) => {
                        new_model.status_message = format!("Failed to load template: {}", e);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::RefreshTemplates => {
                new_model.template.picker.refresh();
                new_model.status_message = "Templates refreshed".to_string();
                (new_model, Cmd::None)
            }

            Message::SetTemplateFocus(focus, mode) => {
                new_model.template.set_focus(focus);
                new_model.template.set_focus_mode(mode);
                if mode == crate::model::template::FocusMode::EditingVariable {
                    new_model
                        .template
                        .variables
                        .move_to_first_missing_variable();
                }
                new_model.status_message = format!("Template focus: {:?} ({:?})", focus, mode);
                (new_model, Cmd::None)
            }

            Message::SetTemplateFocusMode(mode) => {
                new_model.template.set_focus_mode(mode);
                new_model.status_message = format!("Template mode: {:?}", mode);
                (new_model, Cmd::None)
            }

            Message::TemplateEditorInput(key) => {
                new_model.template.editor.editor.input(key);
                new_model.template.editor.sync_content_from_textarea();
                new_model.template.editor.validate_template();
                new_model.template.sync_variables_with_template();
                (new_model, Cmd::None)
            }

            Message::TemplatePickerMove(delta) => {
                if delta > 0 {
                    new_model.template.picker.move_cursor_down();
                } else {
                    new_model.template.picker.move_cursor_up();
                }
                (new_model, Cmd::None)
            }

            Message::VariableStartEditing(var_name) => {
                new_model.template.variables.editing_variable = Some(var_name.clone());
                new_model.template.variables.show_variable_input = true;
                new_model.template.variables.variable_input_content.clear();
                new_model.status_message = format!("Editing variable: {}", var_name);
                (new_model, Cmd::None)
            }

            Message::VariableInputChar(c) => {
                new_model.template.variables.add_char_to_input(c);
                (new_model, Cmd::None)
            }

            Message::VariableInputBackspace => {
                new_model.template.variables.remove_char_from_input();
                (new_model, Cmd::None)
            }

            Message::VariableInputEnter => {
                if let Some((var_name, value)) = new_model.template.variables.finish_editing() {
                    new_model.status_message = format!("Set {} = {}", var_name, value);
                    new_model.template.sync_variables_with_template();
                }
                (new_model, Cmd::None)
            }

            Message::VariableInputCancel => {
                new_model.template.variables.cancel_editing();
                new_model.status_message = "Cancelled variable editing".to_string();
                (new_model, Cmd::None)
            }

            Message::VariableNavigateUp => {
                if new_model.template.variables.cursor > 0 {
                    new_model.template.variables.cursor -= 1;
                }
                (new_model, Cmd::None)
            }

            Message::VariableNavigateDown => {
                let variables = new_model.template.get_organized_variables();
                if new_model.template.variables.cursor < variables.len().saturating_sub(1) {
                    new_model.template.variables.cursor += 1;
                }
                (new_model, Cmd::None)
            }
        }
    }
}
