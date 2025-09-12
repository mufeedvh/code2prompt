//! Template Variable sub-widget.
//!
//! This widget provides a 2-column display for template variables with direct editing.

use crate::model::template::{VariableCategory, VariableInfo, VariableState};
use crate::model::Message;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Template Variable sub-widget
pub struct TemplateVariableWidget;

impl TemplateVariableWidget {
    pub fn new() -> Self {
        Self
    }

    /// Handle key events for the variable widget
    pub fn handle_key_event(
        key: KeyEvent,
        state: &mut VariableState,
        variables: &[VariableInfo],
        is_focused: bool,
    ) -> Option<Message> {
        if !is_focused {
            return None;
        }

        // Handle variable input dialog if active
        if state.is_editing() {
            return Self::handle_variable_input_keys(key, state);
        }

        // Handle navigation and selection
        match key.code {
            KeyCode::Up => {
                state.move_cursor_up(variables.len());
                None
            }
            KeyCode::Down => {
                state.move_cursor_down(variables.len());
                None
            }
            KeyCode::Enter => {
                // Start editing variable if it's user-defined or missing
                if let Some(var_info) = variables.get(state.cursor) {
                    match var_info.category {
                        VariableCategory::User | VariableCategory::Missing => {
                            state.start_editing_variable(
                                var_info.name.clone(),
                                var_info.value.clone(),
                            );
                        }
                        VariableCategory::System => {
                            // System variables cannot be edited - no action
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Handle variable input dialog keys
    fn handle_variable_input_keys(key: KeyEvent, state: &mut VariableState) -> Option<Message> {
        match key.code {
            KeyCode::Esc => {
                state.cancel_editing();
                None
            }
            KeyCode::Enter => {
                state.finish_editing();
                None
            }
            KeyCode::Char(c) => {
                state.add_char_to_input(c);
                None
            }
            KeyCode::Backspace => {
                state.remove_char_from_input();
                None
            }
            _ => None,
        }
    }

    /// Render the variable widget
    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &VariableState,
        variables: &[VariableInfo],
        is_focused: bool,
    ) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Create table-like display with 2 columns
        let mut lines = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::styled(
                "Variable Name",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("                "), // Spacing
            Span::styled(
                "Description/Value",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![Span::raw(
            "────────────────────────────────────────────────────────────────────────────────",
        )]));

        // Variable rows
        for (i, var_info) in variables.iter().enumerate() {
            let is_selected = i == state.cursor && is_focused;

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                match var_info.category {
                    VariableCategory::System => Style::default().fg(Color::Green),
                    VariableCategory::User => Style::default().fg(Color::Cyan),
                    VariableCategory::Missing => Style::default().fg(Color::Red),
                }
            };

            let value_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = match var_info.category {
                VariableCategory::System => "🔧 ",
                VariableCategory::User => "👤 ",
                VariableCategory::Missing => "❌ ",
            };

            let name_part = format!("{}{{{{{}}}}}", prefix, var_info.name);
            let name_padded = format!("{:<24}", name_part);

            let value_part = match var_info.category {
                VariableCategory::System => var_info
                    .description
                    .as_ref()
                    .unwrap_or(&"System variable".to_string())
                    .clone(),
                VariableCategory::User => var_info
                    .value
                    .as_ref()
                    .unwrap_or(&"(empty)".to_string())
                    .clone(),
                VariableCategory::Missing => "⚠️ Not defined".to_string(), // NO "Press Enter to set"
            };

            let line = if is_selected {
                // Highlight entire row for selected item
                Line::from(vec![Span::styled(
                    format!("► {}{}", name_padded, value_part),
                    name_style,
                )])
            } else {
                Line::from(vec![
                    Span::styled(format!("  {}", name_padded), name_style),
                    Span::styled(value_part, value_style),
                ])
            };

            lines.push(line);
        }

        let title_spans = vec![
            Span::styled(
                "v",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("ariables", Style::default().fg(Color::White)),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans))
                    .border_style(border_style),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        Widget::render(paragraph, area, buf);

        // Render variable input popup if active
        if state.is_editing() {
            self.render_variable_input(area, buf, state);
        }
    }

    /// Render variable input popup
    fn render_variable_input(&self, area: Rect, buf: &mut Buffer, state: &VariableState) {
        let popup_area = Self::centered_rect(60, 20, area);
        Clear.render(popup_area, buf);

        let var_name = state
            .get_editing_variable()
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let title = format!("Set Variable: {}", var_name);

        let paragraph = Paragraph::new(state.get_input_content()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        Widget::render(paragraph, popup_area, buf);
    }

    /// Create centered rectangle for popup
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Get help text for the variable widget
    pub fn get_help_text(is_focused: bool, is_editing: bool) -> &'static str {
        if is_editing {
            "Type value | Enter: Save | ESC: Cancel"
        } else if is_focused {
            "↑↓: Navigate | Enter: Edit variable"
        } else {
            "Press 'v' to focus variables"
        }
    }
}

impl Default for TemplateVariableWidget {
    fn default() -> Self {
        Self::new()
    }
}
