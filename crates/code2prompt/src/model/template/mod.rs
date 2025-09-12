//! Template state management module.
//!
//! This module coordinates the three template sub-components:
//! - Editor: Template content editing and validation
//! - Variable: Variable management and validation
//! - Picker: Template selection and loading

pub mod editor;
pub mod picker;
pub mod variable;

pub use editor::EditorState;
pub use picker::{ActiveList, PickerState, TemplateFile};
pub use variable::{VariableCategory, VariableInfo, VariableState};

use crate::model::Message;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Which component is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateFocus {
    Editor,
    Variables,
    Picker,
}

/// Coordinated template state containing all sub-components
#[derive(Debug, Clone)]
pub struct TemplateState {
    pub editor: EditorState,
    pub variables: VariableState,
    pub picker: PickerState,
    pub focus: TemplateFocus,
    pub status_message: String,
}

impl Default for TemplateState {
    fn default() -> Self {
        let mut state = Self {
            editor: EditorState::default(),
            variables: VariableState::default(),
            picker: PickerState::default(),
            focus: TemplateFocus::Editor,
            status_message: String::new(),
        };

        // Initialize variable state with template variables
        state.sync_variables_with_template();
        state
    }
}

impl TemplateState {
    /// Create template state from model (for TUI integration)
    pub fn from_model(model: &crate::model::Model) -> Self {
        // Create a new state based on the model's template state
        model.template.clone()
    }

    /// Synchronize variables with current template content
    pub fn sync_variables_with_template(&mut self) {
        let template_vars = self.editor.get_template_variables();
        self.variables.update_missing_variables(template_vars);
    }

    /// Load a template from the picker
    pub fn load_selected_template(&mut self) -> Result<(), String> {
        if let Some(template) = self.picker.get_selected_template().cloned() {
            let content = self.picker.load_template_content(&template)?;
            self.editor.load_template(content, template.name.clone());
            self.sync_variables_with_template();
            self.status_message = format!("Loaded template: {}", template.name);
            Ok(())
        } else {
            Err("No template selected".to_string())
        }
    }

    /// Set focus to a specific component
    pub fn set_focus(&mut self, focus: TemplateFocus) {
        self.focus = focus;
    }

    /// Get current focus
    pub fn get_focus(&self) -> TemplateFocus {
        self.focus
    }

    /// Check if template is valid for analysis
    pub fn is_valid_for_analysis(&self) -> bool {
        self.editor.is_template_valid() && !self.variables.has_missing_variables()
    }

    /// Get validation message for analysis
    pub fn get_analysis_validation_message(&self) -> String {
        if !self.editor.is_template_valid() {
            format!(
                "Template syntax error: {}",
                self.editor.get_validation_message()
            )
        } else if self.variables.has_missing_variables() {
            self.variables.get_missing_variables_message()
        } else {
            String::new()
        }
    }

    /// Get organized variables for display
    pub fn get_organized_variables(&self) -> Vec<VariableInfo> {
        self.variables
            .get_organized_variables(self.editor.get_template_variables())
    }

    /// Refresh templates in picker
    pub fn refresh_templates(&mut self) {
        self.picker.refresh();
        self.status_message = "Templates refreshed".to_string();
    }

    /// Save current template to custom directory
    pub fn save_template(&self, filename: String) -> Result<(), String> {
        crate::utils::save_template_to_custom_dir(&filename, self.editor.get_content())
            .map(|_| ()) // Ignore the PathBuf return value
            .map_err(|e| format!("Failed to save template: {}", e))
    }

    /// Get current template content for analysis
    pub fn get_template_content(&self) -> &str {
        self.editor.get_content()
    }

    /// Set status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }

    /// Get status message
    pub fn get_status(&self) -> &str {
        &self.status_message
    }

    /// Handle key events for the template system (moved from widget)
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<Message> {
        // Handle variable input dialog first (highest priority)
        if self.variables.is_editing() {
            let variables = self.get_organized_variables();
            let result = crate::widgets::template::TemplateVariableWidget::handle_key_event(
                key,
                &mut self.variables,
                &variables,
                true, // Always focused when editing
            );

            // Update missing variables after variable changes
            if result.is_some() {
                self.sync_variables_with_template();
            }

            return result;
        }

        // Global shortcuts - Focus system (e/v/p)
        match key.code {
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.set_focus(TemplateFocus::Editor);
                return None;
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                self.set_focus(TemplateFocus::Variables);
                return None;
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.set_focus(TemplateFocus::Picker);
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
                if !self.is_valid_for_analysis() {
                    self.set_status(self.get_analysis_validation_message());
                    // Focus on the problematic component
                    if !self.editor.is_template_valid() {
                        self.set_focus(TemplateFocus::Editor);
                    } else if self.variables.has_missing_variables() {
                        self.set_focus(TemplateFocus::Variables);
                    }
                    return None;
                } else {
                    return Some(Message::RunAnalysis);
                }
            }
            _ => {}
        }

        // Handle input based on focused component
        match self.get_focus() {
            TemplateFocus::Editor => {
                let result = crate::widgets::template::TemplateEditorWidget::handle_key_event(
                    key,
                    &mut self.editor,
                    true,
                );

                // Update variables when template content changes
                if result.is_some() {
                    self.sync_variables_with_template();
                }

                result
            }
            TemplateFocus::Variables => {
                let variables = self.get_organized_variables();
                let result = crate::widgets::template::TemplateVariableWidget::handle_key_event(
                    key,
                    &mut self.variables,
                    &variables,
                    true,
                );

                // Update missing variables after variable changes
                if result.is_some() {
                    self.sync_variables_with_template();
                }

                result
            }
            TemplateFocus::Picker => {
                let result = crate::widgets::template::TemplatePickerWidget::handle_key_event(
                    key,
                    &mut self.picker,
                    true,
                );

                // Handle template loading
                if let Some(Message::LoadTemplate) = result {
                    match self.load_selected_template() {
                        Ok(_) => {
                            self.set_focus(TemplateFocus::Editor);
                            None
                        }
                        Err(e) => {
                            self.set_status(e);
                            None
                        }
                    }
                } else if let Some(Message::RefreshTemplates) = result {
                    self.refresh_templates();
                    None
                } else {
                    result
                }
            }
        }
    }

    /// Handle template-related messages
    pub fn handle_message(&mut self, message: &Message) -> Option<Message> {
        match message {
            Message::SaveTemplate(filename) => {
                match self.save_template(filename.clone()) {
                    Ok(_) => {
                        self.status_message = format!("Template saved as: {}", filename);
                        self.refresh_templates(); // Refresh to show new template
                    }
                    Err(e) => {
                        self.status_message = e;
                    }
                }
                None
            }
            Message::ReloadTemplate => {
                // Reload default template
                self.editor = EditorState::default();
                self.sync_variables_with_template();
                self.status_message = "Template reloaded".to_string();
                None
            }
            Message::RunAnalysis => {
                if self.is_valid_for_analysis() {
                    Some(Message::RunAnalysis)
                } else {
                    self.status_message = self.get_analysis_validation_message();
                    None
                }
            }
            _ => None,
        }
    }
}
