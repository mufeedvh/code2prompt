//! Template Editor sub-widget.
//!
//! This widget provides an editable text area for template content with validation.

use crate::model::template::EditorState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

/// Template Editor sub-widget
pub struct TemplateEditorWidget;

impl TemplateEditorWidget {
    pub fn new() -> Self {
        Self
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
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow) // Focused
        } else {
            Style::default().fg(Color::Rgb(139, 69, 19)) // Brown for normal
        };

        // Create title with validation status
        let title_spans = if !state.is_valid {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ditor ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("(SYNTAX ERROR: {})", state.validation_message),
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
                Span::styled("ditor ", Style::default().fg(Color::White)),
                Span::styled(" (MISSING VARIABLES)", Style::default().fg(Color::Red)),
            ]
        } else {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ditor ", Style::default().fg(Color::White)),
                Span::styled(" (VALID)", Style::default().fg(Color::Green)),
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

        // Set text color - always use brown highlight for invalid, white for valid
        if !state.is_valid || has_missing_vars {
            textarea.set_style(Style::default().fg(Color::Rgb(139, 69, 19))); // Brown highlight
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
