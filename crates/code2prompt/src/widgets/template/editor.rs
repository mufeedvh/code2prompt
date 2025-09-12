//! Template Editor sub-widget.
//!
//! This widget provides an editable text area for template content with validation.

use crate::model::template::EditorState;
use crate::model::Message;
use ratatui::{
    crossterm::event::KeyEvent,
    prelude::*,
    widgets::{Block, Borders},
};

/// Template Editor sub-widget
pub struct TemplateEditorWidget;

impl TemplateEditorWidget {
    pub fn new() -> Self {
        Self
    }

    /// Handle key events for the editor
    pub fn handle_key_event(
        key: KeyEvent,
        state: &mut EditorState,
        is_focused: bool,
    ) -> Option<Message> {
        if !is_focused {
            return None;
        }

        // Handle TextArea input when focused
        state.editor.input(key);
        state.sync_content_from_textarea();
        state.validate_template();

        None
    }

    /// Render the template editor
    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut EditorState,
        is_focused: bool,
        has_missing_vars: bool,
    ) {
        // Determine border style based on validation and focus
        let border_style = if !state.is_valid || has_missing_vars {
            Style::default().fg(Color::Red) // Invalid template syntax or missing variables
        } else if is_focused {
            Style::default().fg(Color::Yellow) // Focused and valid
        } else {
            Style::default().fg(Color::Rgb(139, 69, 19)) // Brown for normal/valid
        };

        // Create title with validation status
        let title_spans = if !state.is_valid {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("ditor (SYNTAX ERROR: {})", state.validation_message),
                    Style::default().fg(Color::Red),
                ),
            ]
        } else if has_missing_vars {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ditor (MISSING VARIABLES)", Style::default().fg(Color::Red)),
            ]
        } else {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ditor (VALID)", Style::default().fg(Color::Green)),
            ]
        };

        // Configure TextArea
        let mut textarea = state.editor.clone();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(title_spans))
                .border_style(border_style),
        );

        // Set cursor and text styles based on focus and validation
        if is_focused {
            textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            textarea.set_cursor_style(Style::default().fg(Color::Yellow));
        }

        // Set text color based on validation
        if !state.is_valid || has_missing_vars {
            textarea.set_style(Style::default().fg(Color::LightRed));
        } else {
            textarea.set_style(Style::default().fg(Color::White));
        }

        // Render the TextArea
        Widget::render(&textarea, area, buf);
    }
}

impl Default for TemplateEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}
