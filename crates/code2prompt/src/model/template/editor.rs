//! Template editor state management.
//!
//! This module contains the state and logic for the template editor component,
//! including TextArea management, validation, and content synchronization.

use regex::Regex;
use std::collections::HashSet;
use tui_textarea::TextArea;

/// State for the template editor component
#[derive(Debug)]
pub struct EditorState {
    pub template_content: String,
    pub template_editor: TextArea<'static>,
    pub current_template_name: String,
    pub is_valid: bool,
    pub validation_message: String,
    pub template_variables: Vec<String>, // Variables found in template
}

impl Clone for EditorState {
    fn clone(&self) -> Self {
        let mut new_editor =
            TextArea::from(self.template_editor.lines().iter().map(|s| s.as_str()));
        new_editor.move_cursor(tui_textarea::CursorMove::Jump(
            self.template_editor.cursor().0.try_into().unwrap_or(0),
            self.template_editor.cursor().1.try_into().unwrap_or(0),
        ));

        Self {
            template_content: self.template_content.clone(),
            template_editor: new_editor,
            current_template_name: self.current_template_name.clone(),
            is_valid: self.is_valid,
            validation_message: self.validation_message.clone(),
            template_variables: self.template_variables.clone(),
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        let template_content =
            include_str!("../../../../code2prompt-core/src/default_template_md.hbs").to_string();
        let template_editor = TextArea::from(template_content.lines());

        let mut state = Self {
            template_content: template_content.clone(),
            template_editor,
            current_template_name: "Default (Markdown)".to_string(),
            is_valid: true,
            validation_message: String::new(),
            template_variables: Vec::new(),
        };

        state.analyze_template_variables();
        state
    }
}

impl EditorState {
    /// Create new editor state with specific template content
    pub fn new(content: String, name: String) -> Self {
        let template_editor = TextArea::from(content.lines());

        let mut state = Self {
            template_content: content,
            template_editor,
            current_template_name: name,
            is_valid: true,
            validation_message: String::new(),
            template_variables: Vec::new(),
        };

        state.analyze_template_variables();
        state
    }

    /// Update content from TextArea and re-analyze variables
    pub fn sync_content_from_textarea(&mut self) {
        self.template_content = self.template_editor.lines().join("\n");
        self.analyze_template_variables();
    }

    /// Update TextArea from content string
    pub fn sync_textarea_from_content(&mut self) {
        self.template_editor = TextArea::from(self.template_content.lines());
    }

    /// Parse template content to extract all {{variable}} references
    pub fn analyze_template_variables(&mut self) {
        let re = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
        let mut found_vars = HashSet::new();

        for cap in re.captures_iter(&self.template_content) {
            if let Some(var_name) = cap.get(1) {
                found_vars.insert(var_name.as_str().to_string());
            }
        }

        self.template_variables = found_vars.into_iter().collect();
        self.template_variables.sort();
    }

    /// Get all variables found in the template
    pub fn get_template_variables(&self) -> &[String] {
        &self.template_variables
    }

    /// Validate template syntax (basic validation)
    pub fn validate_template(&mut self) {
        // Basic validation - check for balanced braces
        let open_count = self.template_content.matches("{{").count();
        let close_count = self.template_content.matches("}}").count();

        if open_count != close_count {
            self.is_valid = false;
            self.validation_message = format!(
                "Unbalanced braces: {} opening, {} closing",
                open_count, close_count
            );
        } else {
            self.is_valid = true;
            self.validation_message = String::new();
        }
    }

    /// Load template content from string
    pub fn load_template(&mut self, content: String, name: String) {
        self.template_content = content;
        self.current_template_name = name;
        self.sync_textarea_from_content();
        self.analyze_template_variables();
        self.validate_template();
    }

    /// Get current template content
    pub fn get_content(&self) -> &str {
        &self.template_content
    }

    /// Get current template name
    pub fn get_name(&self) -> &str {
        &self.current_template_name
    }

    /// Check if template is valid
    pub fn is_template_valid(&self) -> bool {
        self.is_valid
    }

    /// Get validation message
    pub fn get_validation_message(&self) -> &str {
        &self.validation_message
    }
}
