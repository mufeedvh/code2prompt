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

    /// Get organized variables for display
    pub fn get_organized_variables(&self) -> Vec<VariableInfo> {
        self.variables
            .get_organized_variables(self.editor.get_template_variables())
    }

    /// Get current template content for analysis
    pub fn get_template_content(&self) -> &str {
        self.editor.get_content()
    }

    /// Get status message
    pub fn get_status(&self) -> &str {
        &self.status_message
    }

    /// Load the currently selected template from the picker
    pub fn load_selected_template(&mut self) -> Result<String, String> {
        let selected_template = self.get_selected_template()?;

        // Load template content based on type
        let (content, template_name) = if selected_template
            .path
            .to_string_lossy()
            .starts_with("builtin://")
        {
            // Load built-in template from embedded resources
            let path_str = selected_template.path.to_string_lossy();
            let template_key = path_str.strip_prefix("builtin://").unwrap_or("");

            if let Some(builtin_template) =
                code2prompt_core::builtin_templates::BuiltinTemplates::get_template(template_key)
            {
                (builtin_template.content, builtin_template.name)
            } else {
                return Err(format!("Built-in template '{}' not found", template_key));
            }
        } else {
            // Load template from file
            let content = std::fs::read_to_string(&selected_template.path)
                .map_err(|e| format!("Failed to read template file: {}", e))?;
            (content, selected_template.name.clone())
        };

        // Update editor with new content
        self.editor.content = content.clone();
        self.editor.current_template_name = template_name.clone();

        // Create new TextArea with the content
        self.editor.editor = tui_textarea::TextArea::from(content.lines());

        // Sync and validate
        self.editor.sync_content_from_textarea();
        self.editor.validate_template();

        Ok(template_name)
    }

    /// Get the currently selected template from the picker
    fn get_selected_template(&self) -> Result<&picker::TemplateFile, String> {
        match self.picker.active_list {
            ActiveList::Default => self
                .picker
                .default_templates
                .get(self.picker.default_cursor)
                .ok_or_else(|| "No default template selected".to_string()),
            ActiveList::Custom => self
                .picker
                .custom_templates
                .get(self.picker.custom_cursor)
                .ok_or_else(|| "No custom template selected".to_string()),
        }
    }
}
