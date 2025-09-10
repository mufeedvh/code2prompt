//! Template state management for the TUI application.
//!
//! This module contains the template state and related functionality
//! for managing template content and editing in the TUI.

use code2prompt_core::template::OutputFormat;

/// Template state containing all template-related data
#[derive(Debug, Clone)]
pub struct TemplateState {
    pub template_content: String,
    pub template_name: String,
    pub template_is_editing: bool,
    pub template_scroll_offset: u16,
    pub template_cursor_position: usize,
}

impl Default for TemplateState {
    fn default() -> Self {
        // Load default markdown template
        let template_content =
            include_str!("../../../code2prompt-core/src/default_template_md.hbs").to_string();

        TemplateState {
            template_content,
            template_name: "Default Template".to_string(),
            template_is_editing: false,
            template_scroll_offset: 0,
            template_cursor_position: 0,
        }
    }
}

impl TemplateState {
    /// Create a new template state with content based on output format
    pub fn new_with_format(output_format: OutputFormat) -> Self {
        let template_content = match output_format {
            OutputFormat::Xml => {
                include_str!("../../../code2prompt-core/src/default_template_xml.hbs").to_string()
            }
            _ => include_str!("../../../code2prompt-core/src/default_template_md.hbs").to_string(),
        };

        TemplateState {
            template_content,
            template_name: "Default Template".to_string(),
            template_is_editing: false,
            template_scroll_offset: 0,
            template_cursor_position: 0,
        }
    }
}
