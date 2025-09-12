//! Template Picker sub-widget.
//!
//! This widget provides template selection with separate default and custom lists.

use crate::model::template::{ActiveList, PickerState, TemplateFile};
use crate::model::Message;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

/// Template Picker sub-widget
pub struct TemplatePickerWidget;

impl TemplatePickerWidget {
    pub fn new() -> Self {
        Self
    }

    /// Handle key events for the picker
    pub fn handle_key_event(
        key: KeyEvent,
        state: &mut PickerState,
        is_focused: bool,
    ) -> Option<Message> {
        if !is_focused {
            return None;
        }

        match key.code {
            KeyCode::Up => {
                state.move_cursor_up();
                None
            }
            KeyCode::Down => {
                state.move_cursor_down();
                None
            }
            KeyCode::Tab => {
                // Switch between default and custom lists
                state.switch_list();
                None
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Char('L') => {
                // Load selected template
                Some(Message::LoadTemplate)
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Refresh templates
                Some(Message::RefreshTemplates)
            }
            _ => None,
        }
    }

    /// Render the template picker with 2 lists
    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &PickerState, is_focused: bool) {
        let _border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Split area into two sections for default and custom templates
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Render default templates section
        self.render_template_list(
            sections[0],
            buf,
            &state.default_templates,
            state.default_cursor,
            state.active_list == ActiveList::Default,
            is_focused,
            "Default Templates",
            "📄",
        );

        // Render custom templates section
        self.render_template_list(
            sections[1],
            buf,
            &state.custom_templates,
            state.custom_cursor,
            state.active_list == ActiveList::Custom,
            is_focused,
            "Custom Templates",
            "📝",
        );
    }

    /// Render a single template list
    fn render_template_list(
        &self,
        area: Rect,
        buf: &mut Buffer,
        templates: &[TemplateFile],
        cursor: usize,
        is_active_list: bool,
        is_widget_focused: bool,
        title: &str,
        icon: &str,
    ) {
        let is_focused = is_widget_focused && is_active_list;

        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else if is_active_list && is_widget_focused {
            Style::default().fg(Color::Cyan) // Indicate this is the active list
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<ListItem> = templates
            .iter()
            .enumerate()
            .map(|(i, template)| {
                let style = if i == cursor && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if i == cursor && is_active_list {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(format!("{} {}", icon, template.name)).style(style)
            })
            .collect();

        // Create title with focus indicators
        let title_spans = if title.contains("Default") {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "p",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("icker - ", Style::default().fg(Color::White)),
                Span::styled(title, Style::default().fg(Color::White)),
                if is_active_list {
                    Span::styled(" (ACTIVE)", Style::default().fg(Color::Cyan))
                } else {
                    Span::styled("", Style::default())
                },
            ]
        } else {
            vec![
                Span::styled(title, Style::default().fg(Color::White)),
                if is_active_list {
                    Span::styled(" (ACTIVE)", Style::default().fg(Color::Cyan))
                } else {
                    Span::styled("", Style::default())
                },
            ]
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans))
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        Widget::render(list, area, buf);
    }

    /// Get help text for the picker
    pub fn get_help_text(is_focused: bool, active_list: ActiveList) -> String {
        if is_focused {
            format!(
                "↑↓: Navigate | Tab: Switch to {} | Enter/l: Load | r: Refresh",
                match active_list {
                    ActiveList::Default => "Custom",
                    ActiveList::Custom => "Default",
                }
            )
        } else {
            "Press 'p' to focus picker".to_string()
        }
    }

    /// Get currently selected template for display
    pub fn get_selected_template_info(state: &PickerState) -> String {
        if let Some(template) = state.get_selected_template() {
            format!(
                "Selected: {} ({})",
                template.name,
                if template.is_default {
                    "Default"
                } else {
                    "Custom"
                }
            )
        } else {
            "No template selected".to_string()
        }
    }
}

impl Default for TemplatePickerWidget {
    fn default() -> Self {
        Self::new()
    }
}
