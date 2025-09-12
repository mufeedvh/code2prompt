//! Template widget module.
//!
//! This module coordinates the three template sub-widgets:
//! - Editor: Template content editing and validation
//! - Variable: Variable management and validation  
//! - Picker: Template selection and loading

pub mod editor;
pub mod picker;
pub mod variable;

pub use editor::TemplateEditorWidget;
pub use picker::TemplatePickerWidget;
pub use variable::TemplateVariableWidget;

use crate::model::template::{TemplateFocus, TemplateState};
use crate::model::{Message, Model};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Main Template widget that coordinates the 3 sub-widgets
pub struct TemplateWidget {
    editor: TemplateEditorWidget,
    variables: TemplateVariableWidget,
    picker: TemplatePickerWidget,
}

impl TemplateWidget {
    pub fn new(_model: &Model) -> Self {
        Self {
            editor: TemplateEditorWidget::new(),
            variables: TemplateVariableWidget::new(),
            picker: TemplatePickerWidget::new(),
        }
    }

    /// Handle key events for the template widget
    pub fn handle_key_event(
        key: KeyEvent,
        _model: &Model,
        state: &mut TemplateState,
    ) -> Option<Message> {
        // Handle variable input dialog first (highest priority)
        if state.variables.is_editing() {
            let variables = state.get_organized_variables();
            let result = TemplateVariableWidget::handle_key_event(
                key,
                &mut state.variables,
                &variables,
                true, // Always focused when editing
            );

            // Update missing variables after variable changes
            if result.is_some() {
                state.sync_variables_with_template();
            }

            return result;
        }

        // Global shortcuts - Focus system (e/v/p)
        match key.code {
            KeyCode::Char('e') | KeyCode::Char('E') => {
                state.set_focus(TemplateFocus::Editor);
                return None;
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                state.set_focus(TemplateFocus::Variables);
                return None;
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                state.set_focus(TemplateFocus::Picker);
                return None;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Save template with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("custom_template_{}", timestamp);
                return Some(Message::SaveTemplate(filename));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reload default template
                return Some(Message::ReloadTemplate);
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Check for missing variables before running analysis - BLOCK if invalid
                if !state.is_valid_for_analysis() {
                    state.set_status(state.get_analysis_validation_message());
                    // Focus on the problematic component
                    if !state.editor.is_template_valid() {
                        state.set_focus(TemplateFocus::Editor);
                    } else if state.variables.has_missing_variables() {
                        state.set_focus(TemplateFocus::Variables);
                    }
                    return None;
                } else {
                    return Some(Message::RunAnalysis);
                }
            }
            _ => {}
        }

        // Handle input based on focused component
        match state.get_focus() {
            TemplateFocus::Editor => {
                let result = TemplateEditorWidget::handle_key_event(key, &mut state.editor, true);

                // Update variables when template content changes
                if result.is_some() {
                    state.sync_variables_with_template();
                }

                result
            }
            TemplateFocus::Variables => {
                let variables = state.get_organized_variables();
                let result = TemplateVariableWidget::handle_key_event(
                    key,
                    &mut state.variables,
                    &variables,
                    true,
                );

                // Update missing variables after variable changes
                if result.is_some() {
                    state.sync_variables_with_template();
                }

                result
            }
            TemplateFocus::Picker => {
                let result = TemplatePickerWidget::handle_key_event(key, &mut state.picker, true);

                // Handle template loading
                if let Some(Message::LoadTemplate) = result {
                    match state.load_selected_template() {
                        Ok(_) => {
                            state.set_focus(TemplateFocus::Editor);
                            None
                        }
                        Err(e) => {
                            state.set_status(e);
                            None
                        }
                    }
                } else if let Some(Message::RefreshTemplates) = result {
                    state.refresh_templates();
                    None
                } else {
                    result
                }
            }
        }
    }

    /// Render the template widget with 3 columns
    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Main layout - content and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Content (3 columns)
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // 3-column layout for content
        self.render_content(chunks[0], buf, state);

        // Footer
        self.render_footer(chunks[1], buf, state);
    }

    /// Render the 3-column content area
    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Flexible 3-column layout
        let min_width = 30;
        let available_width = area.width.saturating_sub(6); // Account for borders

        let constraints = if available_width >= min_width * 3 {
            // Full 3-column layout
            vec![
                Constraint::Percentage(40), // Editor
                Constraint::Percentage(35), // Variables
                Constraint::Percentage(25), // Picker
            ]
        } else if available_width >= min_width * 2 {
            // 2-column layout, hide picker or make it smaller
            vec![
                Constraint::Percentage(60), // Editor
                Constraint::Percentage(40), // Variables
                Constraint::Length(0),      // Picker hidden
            ]
        } else {
            // Single column, show only focused column
            match state.get_focus() {
                TemplateFocus::Editor => vec![
                    Constraint::Percentage(100),
                    Constraint::Length(0),
                    Constraint::Length(0),
                ],
                TemplateFocus::Variables => vec![
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                    Constraint::Length(0),
                ],
                TemplateFocus::Picker => vec![
                    Constraint::Length(0),
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                ],
            }
        };

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Render each column if it has space
        if columns[0].width > 0 {
            let is_editor_focused = state.get_focus() == TemplateFocus::Editor;
            let has_missing_vars = state.variables.has_missing_variables();
            self.editor.render(
                columns[0],
                buf,
                &mut state.editor,
                is_editor_focused,
                has_missing_vars,
            );
        }

        if columns[1].width > 0 {
            let variables = state.get_organized_variables();
            self.variables.render(
                columns[1],
                buf,
                &state.variables,
                &variables,
                state.get_focus() == TemplateFocus::Variables,
            );
        }

        if columns[2].width > 0 {
            self.picker.render(
                columns[2],
                buf,
                &state.picker,
                state.get_focus() == TemplateFocus::Picker,
            );
        }
    }

    /// Render the footer with controls and status
    fn render_footer(&self, area: Rect, buf: &mut Buffer, state: &TemplateState) {
        let footer_text = if !state.get_status().is_empty() {
            state.get_status().to_string()
        } else {
            // Global controls with focus system
            let global_controls =
                "Focus: e(dit) v(ariables) p(icker) | s(ave) r(eload) | Ctrl+Enter: Analyze";

            let specific_controls = match state.get_focus() {
                TemplateFocus::Editor => TemplateEditorWidget::get_help_text(true),
                TemplateFocus::Variables => {
                    TemplateVariableWidget::get_help_text(true, state.variables.is_editing())
                }
                TemplateFocus::Picker => {
                    &TemplatePickerWidget::get_help_text(true, state.picker.active_list)
                }
            };

            format!("{} | {}", global_controls, specific_controls)
        };

        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Gray));
        footer.render(area, buf);
    }
}

impl StatefulWidget for TemplateWidget {
    type State = TemplateState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TemplateWidget::render(&self, area, buf, state);
    }
}
