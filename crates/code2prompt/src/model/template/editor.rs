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
    pub content: String,
    pub editor: TextArea<'static>,
    pub current_template_name: String,
    pub is_valid: bool,
    pub validation_message: String,
    pub template_variables: Vec<String>, // Variables found in template
}

impl Clone for EditorState {
    fn clone(&self) -> Self {
        let mut new_editor = TextArea::from(self.editor.lines().iter().map(|s| s.as_str()));
        new_editor.move_cursor(tui_textarea::CursorMove::Jump(
            self.editor.cursor().0.try_into().unwrap_or(0),
            self.editor.cursor().1.try_into().unwrap_or(0),
        ));

        Self {
            content: self.content.clone(),
            editor: new_editor,
            current_template_name: self.current_template_name.clone(),
            is_valid: self.is_valid,
            validation_message: self.validation_message.clone(),
            template_variables: self.template_variables.clone(),
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        let content =
            include_str!("../../../../code2prompt-core/src/default_template_md.hbs").to_string();
        let editor = TextArea::from(content.lines());

        let mut state = Self {
            content: content.clone(),
            editor,
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
    /// Update content from TextArea and re-analyze variables
    pub fn sync_content_from_textarea(&mut self) {
        self.content = self.editor.lines().join("\n");
        self.analyze_template_variables();
    }

    /// Update TextArea from content string
    pub fn sync_textarea_from_content(&mut self) {
        self.editor = TextArea::from(self.content.lines());
    }

    /// Parse template content to extract all {{variable}} references
    pub fn analyze_template_variables(&mut self) {
        let re = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
        let mut found_vars = HashSet::new();

        for cap in re.captures_iter(&self.content) {
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

    /// Validate template syntax with enhanced Handlebars checking
    pub fn validate_template(&mut self) {
        // First check for balanced braces
        let open_count = self.content.matches("{{").count();
        let close_count = self.content.matches("}}").count();

        if open_count != close_count {
            self.is_valid = false;
            self.validation_message = format!(
                "Unbalanced braces: {} opening, {} closing",
                open_count, close_count
            );
            return;
        }

        // Try to compile the template with Handlebars
        match self.compile_template() {
            Ok(_) => {
                self.is_valid = true;
                self.validation_message = String::new();
            }
            Err(e) => {
                self.is_valid = false;
                self.validation_message = format!("Template syntax error: {}", e);
            }
        }
    }

    /// Attempt to compile the template to check for syntax errors
    fn compile_template(&self) -> Result<(), String> {
        let mut handlebars = handlebars::Handlebars::new();

        // Set strict mode to catch undefined variables
        handlebars.set_strict_mode(false); // Allow undefined variables for now

        match handlebars.register_template_string("test", &self.content) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Load template content from string
    pub fn load_template(&mut self, content: String, name: String) {
        self.content = content;
        self.current_template_name = name;
        self.sync_textarea_from_content();
        self.analyze_template_variables();
        self.validate_template();
    }

    /// Get current template content
    pub fn get_content(&self) -> &str {
        &self.content
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
