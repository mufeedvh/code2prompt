//! Template picker state management.
//!
//! This module contains the state and logic for the template picker component,
//! including loading templates from default and custom directories.

use std::path::PathBuf;

/// Represents a template file
#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub name: String,
    pub path: PathBuf,
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
        });

        self.default_templates.push(TemplateFile {
            name: "Default (XML)".to_string(),
            path: PathBuf::new(),
        });

        // Load templates from default directory using utility function
        if let Ok(all_templates) = crate::utils::load_all_templates() {
            for (name, path, is_builtin) in all_templates {
                if is_builtin {
                    self.default_templates.push(TemplateFile { name, path });
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
                    self.custom_templates.push(TemplateFile { name, path });
                }
            }
        }
    }

    /// Move cursor up in unified list
    pub fn move_cursor_up(&mut self) {
        let total_items = self.get_total_selectable_items();
        if total_items == 0 {
            return;
        }

        let current_global = self.get_global_template_index();
        let new_global = if current_global == 0 {
            total_items - 1 // Wrap to bottom
        } else {
            current_global - 1
        };

        self.set_cursor_from_global_position(new_global);
    }

    /// Move cursor down in unified list
    pub fn move_cursor_down(&mut self) {
        let total_items = self.get_total_selectable_items();
        if total_items == 0 {
            return;
        }

        let current_global = self.get_global_template_index();
        let new_global = (current_global + 1) % total_items;

        self.set_cursor_from_global_position(new_global);
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

    /// Get global cursor position for unified list display (for rendering)
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

    /// Get global template index (for navigation logic)
    fn get_global_template_index(&self) -> usize {
        match self.active_list {
            ActiveList::Default => self.default_cursor,
            ActiveList::Custom => self.default_templates.len() + self.custom_cursor,
        }
    }

    /// Get total number of selectable items (templates only, not headers)
    fn get_total_selectable_items(&self) -> usize {
        self.default_templates.len() + self.custom_templates.len()
    }

    /// Set cursor position from global position in unified list
    fn set_cursor_from_global_position(&mut self, global_pos: usize) {
        let mut template_index = 0;

        // Check if position is in default templates
        if global_pos < self.default_templates.len() {
            self.active_list = ActiveList::Default;
            self.default_cursor = global_pos;
            return;
        }
        template_index += self.default_templates.len();

        // Check if position is in custom templates
        if global_pos < template_index + self.custom_templates.len() {
            self.active_list = ActiveList::Custom;
            self.custom_cursor = global_pos - template_index;
        }
    }
}
