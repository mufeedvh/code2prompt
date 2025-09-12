//! Template picker state management.
//!
//! This module contains the state and logic for the template picker component,
//! including loading templates from default and custom directories.

use std::fs;
use std::path::PathBuf;

/// Represents a template file
#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
}

/// Which list is currently active in the picker
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveList {
    Default,
    Custom,
}

/// State for the template picker component
#[derive(Debug, Clone)]
pub struct PickerState {
    pub default_templates: Vec<TemplateFile>,
    pub custom_templates: Vec<TemplateFile>,
    pub active_list: ActiveList,
    pub default_cursor: usize,
    pub custom_cursor: usize,
}

impl Default for PickerState {
    fn default() -> Self {
        let mut state = Self {
            default_templates: Vec::new(),
            custom_templates: Vec::new(),
            active_list: ActiveList::Default,
            default_cursor: 0,
            custom_cursor: 0,
        };

        state.load_all_templates();
        state
    }
}

impl PickerState {
    /// Load all templates from default and custom directories
    pub fn load_all_templates(&mut self) {
        self.load_default_templates();
        self.load_custom_templates();
    }

    /// Load built-in default templates
    fn load_default_templates(&mut self) {
        self.default_templates.clear();

        // Add built-in templates
        self.default_templates.push(TemplateFile {
            name: "Default (Markdown)".to_string(),
            path: PathBuf::new(),
            is_default: true,
        });

        self.default_templates.push(TemplateFile {
            name: "Default (XML)".to_string(),
            path: PathBuf::new(),
            is_default: true,
        });

        // Load templates from default directory using utility function
        if let Ok(all_templates) = crate::utils::load_all_templates() {
            for (name, path, is_builtin) in all_templates {
                if is_builtin {
                    self.default_templates.push(TemplateFile {
                        name,
                        path,
                        is_default: true,
                    });
                }
            }
        }
    }

    /// Load custom templates from user directory
    fn load_custom_templates(&mut self) {
        self.custom_templates.clear();

        // Load templates from custom directory using utility function
        if let Ok(all_templates) = crate::utils::load_all_templates() {
            for (name, path, is_builtin) in all_templates {
                if !is_builtin {
                    self.custom_templates.push(TemplateFile {
                        name,
                        path,
                        is_default: false,
                    });
                }
            }
        }
    }

    /// Get currently active templates list
    pub fn get_active_templates(&self) -> &[TemplateFile] {
        match self.active_list {
            ActiveList::Default => &self.default_templates,
            ActiveList::Custom => &self.custom_templates,
        }
    }

    /// Get current cursor position for active list
    pub fn get_active_cursor(&self) -> usize {
        match self.active_list {
            ActiveList::Default => self.default_cursor,
            ActiveList::Custom => self.custom_cursor,
        }
    }

    /// Get currently selected template
    pub fn get_selected_template(&self) -> Option<&TemplateFile> {
        let templates = self.get_active_templates();
        let cursor = self.get_active_cursor();
        templates.get(cursor)
    }

    /// Move cursor up in active list
    pub fn move_cursor_up(&mut self) {
        match self.active_list {
            ActiveList::Default => {
                if self.default_cursor > 0 {
                    self.default_cursor -= 1;
                } else if !self.default_templates.is_empty() {
                    self.default_cursor = self.default_templates.len() - 1; // Wrap to bottom
                }
            }
            ActiveList::Custom => {
                if self.custom_cursor > 0 {
                    self.custom_cursor -= 1;
                } else if !self.custom_templates.is_empty() {
                    self.custom_cursor = self.custom_templates.len() - 1; // Wrap to bottom
                }
            }
        }
    }

    /// Move cursor down in active list
    pub fn move_cursor_down(&mut self) {
        match self.active_list {
            ActiveList::Default => {
                if !self.default_templates.is_empty() {
                    self.default_cursor = (self.default_cursor + 1) % self.default_templates.len();
                }
            }
            ActiveList::Custom => {
                if !self.custom_templates.is_empty() {
                    self.custom_cursor = (self.custom_cursor + 1) % self.custom_templates.len();
                }
            }
        }
    }

    /// Switch between default and custom lists
    pub fn switch_list(&mut self) {
        self.active_list = match self.active_list {
            ActiveList::Default => ActiveList::Custom,
            ActiveList::Custom => ActiveList::Default,
        };
    }

    /// Load template content from file
    pub fn load_template_content(&self, template: &TemplateFile) -> Result<String, String> {
        if template.is_default && template.path.as_os_str().is_empty() {
            // Built-in templates
            match template.name.as_str() {
                "Default (Markdown)" => Ok(include_str!(
                    "../../../../code2prompt-core/src/default_template_md.hbs"
                )
                .to_string()),
                "Default (XML)" => Ok(include_str!(
                    "../../../../code2prompt-core/src/default_template_xml.hbs"
                )
                .to_string()),
                _ => Err("Unknown default template".to_string()),
            }
        } else {
            // File-based templates
            match fs::read_to_string(&template.path) {
                Ok(content) => Ok(content),
                Err(e) => Err(format!("Failed to load template: {}", e)),
            }
        }
    }

    /// Refresh templates by reloading from directories
    pub fn refresh(&mut self) {
        self.load_all_templates();

        // Reset cursors if they're out of bounds
        if self.default_cursor >= self.default_templates.len() {
            self.default_cursor = self.default_templates.len().saturating_sub(1);
        }
        if self.custom_cursor >= self.custom_templates.len() {
            self.custom_cursor = self.custom_templates.len().saturating_sub(1);
        }
    }

    /// Get global cursor position for unified list display
    pub fn get_global_cursor_position(&self) -> usize {
        let mut position = 0;

        // Count default templates section
        if !self.default_templates.is_empty() {
            position += 1; // Section header
            if self.active_list == ActiveList::Default {
                position += self.default_cursor;
                return position;
            }
            position += self.default_templates.len();
        }

        // Count custom templates section
        if !self.custom_templates.is_empty() {
            if !self.default_templates.is_empty() {
                position += 1; // Separator
            }
            position += 1; // Section header
            if self.active_list == ActiveList::Custom {
                position += self.custom_cursor;
                return position;
            }
        }

        position
    }

    /// Get currently selected template from unified list
    pub fn get_selected_template_from_global_position(&self) -> Option<&TemplateFile> {
        match self.active_list {
            ActiveList::Default => self.default_templates.get(self.default_cursor),
            ActiveList::Custom => self.custom_templates.get(self.custom_cursor),
        }
    }
}
