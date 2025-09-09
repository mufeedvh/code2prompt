//! Statistics token map widget for displaying token distribution.

use crate::model::{Message, Model};
use crate::token_map::{format_token_map_for_tui, TuiColor};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// State for the token map widget
#[derive(Debug, Clone)]
pub struct TokenMapState {
    pub scroll_position: u16,
}

impl TokenMapState {
    pub fn from_model(model: &Model) -> Self {
        Self {
            scroll_position: model.statistics_scroll,
        }
    }
}

/// Widget for token map display
pub struct StatisticsTokenMapWidget<'a> {
    pub model: &'a Model,
}

impl<'a> StatisticsTokenMapWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Handle key events for token map
    pub fn handle_key_event(key: KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => Some(Message::RunAnalysis),
            KeyCode::Left => Some(Message::CycleStatisticsView(-1)), // Previous view
            KeyCode::Right => Some(Message::CycleStatisticsView(1)), // Next view
            KeyCode::Up => Some(Message::ScrollStatistics(-1)),
            KeyCode::Down => Some(Message::ScrollStatistics(1)),
            KeyCode::PageUp => Some(Message::ScrollStatistics(-5)),
            KeyCode::PageDown => Some(Message::ScrollStatistics(5)),
            KeyCode::Home => Some(Message::ScrollStatistics(-9999)),
            KeyCode::End => Some(Message::ScrollStatistics(9999)),
            _ => None,
        }
    }
}

impl<'a> StatefulWidget for StatisticsTokenMapWidget<'a> {
    type State = TokenMapState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Token map content
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        let title = "🗂️  Token Map";

        if self.model.token_map_entries.is_empty() {
            let placeholder_text = if self.model.generated_prompt.is_some() {
                "🗂️  Token Map View\n\nNo token map data available.\n\nPress Enter to re-run analysis."
            } else {
                "🗂️  Token Map View\n\nRun analysis first to see token distribution.\n\nPress Enter to analyze selected files."
            };

            let placeholder_widget = Paragraph::new(placeholder_text)
                .block(Block::default().borders(Borders::ALL).title(title))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Gray));

            Widget::render(placeholder_widget, layout[0], buf);

            // Instructions
            let instructions =
                Paragraph::new("Enter: Run Analysis | ←→: Switch View | Tab/Shift+Tab: Switch Tab")
                    .block(Block::default().borders(Borders::ALL).title("Controls"))
                    .style(Style::default().fg(Color::Gray));
            Widget::render(instructions, layout[1], buf);
            return;
        }

        // Use the shared token map formatting logic from token_map.rs with adaptive layout
        let total_tokens = self.model.token_count.unwrap_or(0);
        let terminal_width = area.width as usize;
        let formatted_lines =
            format_token_map_for_tui(&self.model.token_map_entries, total_tokens, terminal_width);

        // Calculate viewport for scrolling
        let content_height = layout[0].height.saturating_sub(2) as usize; // Account for borders
        let scroll_start = state.scroll_position as usize;
        let scroll_end = (scroll_start + content_height).min(formatted_lines.len());

        // Convert formatted lines to ListItems with proper column layout and filename coloring
        let items: Vec<ListItem> = formatted_lines
            .iter()
            .skip(scroll_start)
            .take(content_height)
            .map(|line| {
                // Convert TuiColor to ratatui Color for filename only
                let name_color = match line.name_color {
                    TuiColor::White => Color::White,
                    TuiColor::Gray => Color::Gray,
                    TuiColor::Red => Color::Red,
                    TuiColor::Green => Color::Green,
                    TuiColor::Blue => Color::Blue,
                    TuiColor::Yellow => Color::Yellow,
                    TuiColor::Cyan => Color::Cyan,
                    TuiColor::Magenta => Color::Magenta,
                    TuiColor::LightRed => Color::LightRed,
                    TuiColor::LightGreen => Color::LightGreen,
                    TuiColor::LightBlue => Color::LightBlue,
                    TuiColor::LightYellow => Color::LightYellow,
                    TuiColor::LightCyan => Color::LightCyan,
                    TuiColor::LightMagenta => Color::LightMagenta,
                };

                // Create spans with proper coloring - only filename gets color, rest is white
                let spans = vec![
                    Span::styled(&line.tokens_part, Style::default().fg(Color::White)),
                    Span::styled("   ", Style::default().fg(Color::White)), // spacing
                    Span::styled(&line.prefix_part, Style::default().fg(Color::White)),
                    Span::styled(&line.name_part, Style::default().fg(name_color)), // Only filename colored
                    Span::styled(" ", Style::default().fg(Color::White)),           // spacing
                    Span::styled(&line.bar_part, Style::default().fg(Color::White)),
                    Span::styled(" ", Style::default().fg(Color::White)), // spacing
                    Span::styled(&line.percentage_part, Style::default().fg(Color::White)),
                ];

                ListItem::new(Line::from(spans))
            })
            .collect();

        // Create title with scroll indicator
        let scroll_title = if formatted_lines.len() > content_height {
            format!(
                "{} | Showing {}-{} of {}",
                title,
                scroll_start + 1,
                scroll_end,
                formatted_lines.len()
            )
        } else {
            title.to_string()
        };

        let token_map_widget =
            List::new(items).block(Block::default().borders(Borders::ALL).title(scroll_title));

        Widget::render(token_map_widget, layout[0], buf);

        // Instructions
        let instructions = Paragraph::new("Enter: Run Analysis | ←→: Switch View | ↑↓/PgUp/PgDn: Scroll | Tab/Shift+Tab: Switch Tab")
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Gray));
        Widget::render(instructions, layout[1], buf);
    }
}
