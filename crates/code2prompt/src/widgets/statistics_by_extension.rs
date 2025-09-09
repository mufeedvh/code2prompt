//! Statistics by extension widget for displaying extension-based histogram.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// State for the extension statistics widget
#[derive(Debug, Clone)]
pub struct ExtensionState {
    pub scroll_position: u16,
}

impl ExtensionState {
    pub fn from_model(model: &Model) -> Self {
        Self {
            scroll_position: model.statistics_scroll,
        }
    }
}

/// Widget for extension-based statistics display
pub struct StatisticsByExtensionWidget<'a> {
    pub model: &'a Model,
}

impl<'a> StatisticsByExtensionWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Handle key events for extension statistics
    pub fn handle_key_event(key: KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => Some(Message::RunAnalysis),
            KeyCode::Up => Some(Message::ScrollStatistics(-1)),
            KeyCode::Down => Some(Message::ScrollStatistics(1)),
            KeyCode::PageUp => Some(Message::ScrollStatistics(-5)),
            KeyCode::PageDown => Some(Message::ScrollStatistics(5)),
            KeyCode::Home => Some(Message::ScrollStatistics(-9999)),
            KeyCode::End => Some(Message::ScrollStatistics(9999)),
            _ => None,
        }
    }

    /// Format number according to token format setting
    fn format_number(
        num: usize,
        token_format: &code2prompt_core::tokenizer::TokenFormat,
    ) -> String {
        use code2prompt_core::tokenizer::TokenFormat;
        use num_format::{SystemLocale, ToFormattedString};

        match token_format {
            TokenFormat::Format => SystemLocale::default()
                .map(|locale| num.to_formatted_string(&locale))
                .unwrap_or_else(|_| num.to_string()),
            TokenFormat::Raw => num.to_string(),
        }
    }
}

impl<'a> StatefulWidget for StatisticsByExtensionWidget<'a> {
    type State = ExtensionState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Extension statistics content
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        let title = "📁 By Extension";

        if self.model.token_map_entries.is_empty() {
            let placeholder_text = if self.model.generated_prompt.is_some() {
                "📁 Extensions View\n\nNo token map data available.\n\nPress Enter to re-run analysis."
            } else {
                "📁 Extensions View\n\nRun analysis first to see token breakdown by file extension.\n\nPress Enter to analyze selected files."
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

        // Aggregate tokens by file extension
        let mut extension_stats: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();
        let total_tokens = self.model.token_count.unwrap_or(0);

        for entry in &self.model.token_map_entries {
            if !entry.metadata.is_dir {
                let extension = entry
                    .name
                    .split('.')
                    .next_back()
                    .map(|ext| format!(".{}", ext))
                    .unwrap_or_else(|| "(no extension)".to_string());

                let (tokens, count) = extension_stats.entry(extension).or_insert((0, 0));
                *tokens += entry.tokens;
                *count += 1;
            }
        }

        // Convert to sorted vec
        let mut ext_vec: Vec<(String, usize, usize)> = extension_stats
            .into_iter()
            .map(|(ext, (tokens, count))| (ext, tokens, count))
            .collect();
        ext_vec.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by tokens desc

        // Calculate viewport for scrolling
        let content_height = layout[0].height.saturating_sub(2) as usize;
        let scroll_start = state.scroll_position as usize;
        let scroll_end = (scroll_start + content_height).min(ext_vec.len());

        // Create list items
        let items: Vec<ListItem> = ext_vec
            .iter()
            .skip(scroll_start)
            .take(content_height)
            .map(|(extension, tokens, count)| {
                let percentage = if total_tokens > 0 {
                    (*tokens as f64 / total_tokens as f64) * 100.0
                } else {
                    0.0
                };

                // Create visual bar
                let bar_width: usize = 25;
                let filled_chars = ((percentage / 100.0) * bar_width as f64) as usize;
                let bar = format!(
                    "{}{}",
                    "█".repeat(filled_chars),
                    "░".repeat(bar_width.saturating_sub(filled_chars))
                );

                // Choose color based on extension
                let color = match extension.as_str() {
                    ".rs" => Color::LightRed,
                    ".md" | ".txt" | ".rst" => Color::Green,
                    ".toml" | ".json" | ".yaml" | ".yml" => Color::Magenta,
                    ".js" | ".ts" | ".jsx" | ".tsx" => Color::Cyan,
                    ".py" => Color::LightYellow,
                    ".go" => Color::LightBlue,
                    ".java" | ".kt" => Color::Red,
                    ".cpp" | ".c" | ".h" => Color::Blue,
                    _ => Color::White,
                };

                let content = format!(
                    "{:<12} │{}│ {:>6} ({:>4.1}%) | {} files",
                    extension,
                    bar,
                    Self::format_number(*tokens, &self.model.session.config.token_format),
                    percentage,
                    count
                );

                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        // Create title with scroll indicator
        let scroll_title = if ext_vec.len() > content_height {
            format!(
                "{} | Showing {}-{} of {}",
                title,
                scroll_start + 1,
                scroll_end,
                ext_vec.len()
            )
        } else {
            title.to_string()
        };

        let extensions_widget = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(scroll_title))
            .style(Style::default().fg(Color::White));

        Widget::render(extensions_widget, layout[0], buf);

        // Instructions
        let instructions = Paragraph::new("Enter: Run Analysis | ←→: Switch View | ↑↓/PgUp/PgDn: Scroll | Tab/Shift+Tab: Switch Tab")
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Gray));
        Widget::render(instructions, layout[1], buf);
    }
}
