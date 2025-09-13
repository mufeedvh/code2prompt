//! Template Picker sub-widget.
//!
//! This widget provides template selection with separate default and custom lists.

use crate::model::template::{ActiveList, PickerState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

/// Template Picker sub-widget
pub struct TemplatePickerWidget;

impl TemplatePickerWidget {
    pub fn new() -> Self {
        Self
    }

    /// Render the template picker as a single unified list with groups
    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &PickerState, is_focused: bool) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Create unified list with section headers
        let mut items = Vec::new();
        let mut item_index = 0;
        let global_cursor = state.get_global_cursor_position();

        // Default Templates Section
        if !state.default_templates.is_empty() {
            // Section header
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📄 ", Style::default().fg(Color::White)),
                Span::styled(
                    "Default Templates",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
            item_index += 1;

            // Default template items
            for template in state.default_templates.iter() {
                let is_selected = global_cursor == item_index;
                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected { "► " } else { "  " };
                items.push(ListItem::new(format!("{}📄 {}", prefix, template.name)).style(style));
                item_index += 1;
            }
        }

        // Custom Templates Section
        if !state.custom_templates.is_empty() {
            // Add separator if we have default templates
            if !state.default_templates.is_empty() {
                items.push(ListItem::new(""));
                item_index += 1;
            }

            // Section header
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📝 ", Style::default().fg(Color::White)),
                Span::styled(
                    "Custom Templates",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
            item_index += 1;

            // Custom template items
            for template in state.custom_templates.iter() {
                let is_selected = global_cursor == item_index;
                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected { "► " } else { "  " };
                items.push(ListItem::new(format!("{}📝 {}", prefix, template.name)).style(style));
                item_index += 1;
            }
        }

        // Create title with focus indicators
        let title_spans = vec![
            Span::styled("Template ", Style::default().fg(Color::White)),
            Span::styled(
                "p",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("icker", Style::default().fg(Color::White)),
        ];

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(title_spans))
                .border_style(border_style),
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
}

impl Default for TemplatePickerWidget {
    fn default() -> Self {
        Self::new()
    }
}
