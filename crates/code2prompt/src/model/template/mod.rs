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
pub use picker::{ActiveList, PickerState};
pub use variable::{VariableCategory, VariableInfo, VariableState};

use crate::model::Message;

/// Which component is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateFocus {
    Editor,
    Variables,
    Picker,
}

/// Focus mode determines interaction behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusMode {
    Normal,          // Can switch between panels with e/v/p
    EditingTemplate, // Locked to editor, ESC to exit
    EditingVariable, // Locked to variables, ESC to exit
}

/// Coordinated template state containing all sub-components
#[derive(Debug, Clone)]
pub struct TemplateState {
    pub editor: EditorState,
    pub variables: VariableState,
    pub picker: PickerState,
    pub focus: TemplateFocus,
    pub focus_mode: FocusMode,
    pub status_message: String,
}

impl Default for TemplateState {
    fn default() -> Self {
        let mut state = Self {
            editor: EditorState::default(),
            variables: VariableState::default(),
            picker: PickerState::default(),
            focus: TemplateFocus::Editor,
            focus_mode: FocusMode::Normal,
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

    /// Set focus to a specific component
    pub fn set_focus(&mut self, focus: TemplateFocus) {
        self.focus = focus;
    }

    /// Get current focus
    pub fn get_focus(&self) -> TemplateFocus {
        self.focus
    }

    /// Set focus mode
    pub fn set_focus_mode(&mut self, mode: FocusMode) {
        self.focus_mode = mode;
    }

    /// Get current focus mode
    pub fn get_focus_mode(&self) -> FocusMode {
        self.focus_mode
    }

    /// Check if currently in an editing mode
    pub fn is_in_editing_mode(&self) -> bool {
        matches!(
            self.focus_mode,
            FocusMode::EditingTemplate | FocusMode::EditingVariable
        )
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

    /// Get status message
    pub fn get_status(&self) -> &str {
        &self.status_message
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
