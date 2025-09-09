//! Settings widget for configuration management.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// State for the settings widget
#[derive(Debug, Clone)]
pub struct SettingsState {
    pub settings_cursor: usize,
}

impl SettingsState {
    pub fn from_model(model: &Model) -> Self {
        Self {
            settings_cursor: model.settings_cursor,
        }
    }
}

/// Widget for settings configuration
pub struct SettingsWidget<'a> {
    pub model: &'a Model,
}

impl<'a> SettingsWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Handle key events for settings
    pub fn handle_key_event(key: KeyEvent, model: &Model) -> Option<Message> {
        match key.code {
            KeyCode::Up => Some(Message::MoveSettingsCursor(-1)),
            KeyCode::Down => Some(Message::MoveSettingsCursor(1)),
            KeyCode::Char(' ') => Some(Message::ToggleSetting(model.settings_cursor)),
            KeyCode::Left | KeyCode::Right => Some(Message::CycleSetting(model.settings_cursor)),
            KeyCode::Enter => Some(Message::RunAnalysis),
            _ => None,
        }
    }
}

impl<'a> StatefulWidget for SettingsWidget<'a> {
    type State = SettingsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let settings_groups = self.model.get_settings_groups();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Settings list
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Build grouped settings display
        let mut items: Vec<ListItem> = Vec::new();
        let mut item_index = 0;

        for group in &settings_groups {
            // Group header
            items.push(
                ListItem::new(format!("── {} ──", group.name)).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            );

            // Group items
            for item in &group.items {
                let value_display = match &item.setting_type {
                    crate::model::SettingType::Boolean(val) => {
                        if *val {
                            "[●] ON".to_string()
                        } else {
                            "[○] OFF".to_string()
                        }
                    }
                    crate::model::SettingType::Choice { options, selected } => {
                        let current = options.get(*selected).cloned().unwrap_or_default();
                        let total = options.len();
                        format!("[▼ {} ({}/{})]", current, selected + 1, total)
                    }
                };

                // Better aligned layout: Name (20 chars) | Value (15 chars) | Description
                let content = format!(
                    "  {:<20} {:<15} {}",
                    item.name, value_display, item.description
                );
                let mut style = Style::default();

                if item_index == state.settings_cursor {
                    style = style
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD);
                }

                // Color based on setting type
                match &item.setting_type {
                    crate::model::SettingType::Boolean(true) => {
                        style = style.fg(Color::Green);
                    }
                    crate::model::SettingType::Boolean(false) => {
                        style = style.fg(Color::Red);
                    }
                    crate::model::SettingType::Choice { .. } => {
                        style = style.fg(Color::Cyan);
                    }
                }

                items.push(ListItem::new(content).style(style));
                item_index += 1;
            }

            // Add spacing between groups
            items.push(ListItem::new(""));
        }

        let settings_widget = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Settings"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        Widget::render(settings_widget, layout[0], buf);

        // Instructions
        let instructions = Paragraph::new(
            "↑↓: Navigate | Space: Toggle | ←→: Cycle Options | Enter: Run Analysis",
        )
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
        Widget::render(instructions, layout[1], buf);
    }
}
