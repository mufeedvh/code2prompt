//! Statistics by extension widget for displaying extension-based histogram.

use crate::model::{Model, StatisticsState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// State for the extension statistics widget - eliminated redundant state
pub type ExtensionState = ();

/// Widget for extension-based statistics display
pub struct StatisticsByExtensionWidget<'a> {
    pub model: &'a Model,
}

impl<'a> StatisticsByExtensionWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl<'a> StatefulWidget for StatisticsByExtensionWidget<'a> {
    type State = ExtensionState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Extension statistics content
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        let title = "📁 By Extension";

        if self.model.statistics.token_map_entries.is_empty() {
            let placeholder_text = if self.model.prompt_output.generated_prompt.is_some() {
                "\nNo token map data available.\n\nPress Enter to re-run analysis."
            } else {
                "\nRun analysis first to see token breakdown by file extension.\n\nPress Enter to run analysis."
            };

            let placeholder_widget = Paragraph::new(placeholder_text)
                .block(Block::default().borders(Borders::ALL).title(title))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            Widget::render(placeholder_widget, layout[0], buf);

            // Instructions
            let instructions =
                Paragraph::new("Enter: Run Analysis | ←→: Switch View | Tab/Shift+Tab: Switch Tab")
                    .block(Block::default().borders(Borders::ALL).title("Controls"))
                    .style(Style::default().fg(Color::Gray));
            Widget::render(instructions, layout[1], buf);
            return;
        }

        // Use business logic from Model - pure Elm/Redux pattern
        let ext_vec = self.model.statistics.aggregate_by_extension();
        let total_tokens = self.model.prompt_output.token_count.unwrap_or(0);

        // Calculate viewport for scrolling - read directly from Model
        let content_height = layout[0].height.saturating_sub(2).max(1) as usize;
        let total = ext_vec.len();
        let max_scroll = total.saturating_sub(content_height);
        let scroll_start = (self.model.statistics.scroll as usize).min(max_scroll);
        let scroll_end = (scroll_start + content_height).min(total);

        // Calculate dynamic column widths based on available space and content
        let available_width = layout[0].width.saturating_sub(4) as usize; // Account for borders and padding

        // Calculate maximum widths needed for each column
        let max_ext_width = ext_vec
            .iter()
            .map(|(ext, _, _)| ext.len())
            .max()
            .unwrap_or(12)
            .max(12); // Minimum 12 chars for "Extension"

        let max_tokens_width = ext_vec
            .iter()
            .map(|(_, tokens, _)| {
                StatisticsState::format_number(*tokens, &self.model.session.config.token_format)
                    .len()
            })
            .max()
            .unwrap_or(6)
            .max(6); // Minimum 6 chars for tokens

        let max_count_width = ext_vec
            .iter()
            .map(|(_, _, count)| count.to_string().len())
            .max()
            .unwrap_or(3)
            .max(3); // Minimum 3 chars for count

        // Fixed widths for percentage and separators
        let percentage_width = 7; // "(100.0%)"
        let separators_width = 8; // " │ │ " + " | " + " files"

        // Calculate remaining space for the progress bar
        let fixed_content_width = max_ext_width
            + max_tokens_width
            + percentage_width
            + max_count_width
            + separators_width
            + 5; // +5 for "files"
        let bar_width = if available_width > fixed_content_width {
            (available_width - fixed_content_width).clamp(10, 40) // Between 10 and 40 chars
        } else {
            15 // Fallback minimum bar width
        };

        // Create list items with dynamic formatting
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

                // Create visual bar with calculated width
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

                // Format with dynamic column widths
                let formatted_tokens = StatisticsState::format_number(
                    *tokens,
                    &self.model.session.config.token_format,
                );
                let content = format!(
                    "{:<width_ext$} │{}│ {:>width_tokens$} ({:>4.1}%) | {:>width_count$} files",
                    extension,
                    bar,
                    formatted_tokens,
                    percentage,
                    count,
                    width_ext = max_ext_width,
                    width_tokens = max_tokens_width,
                    width_count = max_count_width
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

        // Add header row for better column alignment
        let header = format!(
            "{:<width_ext$} │{:^width_bar$}│ {:>width_tokens$} {:>7} | {:>width_count$} Files",
            "Extension",
            "Usage",
            "Tokens",
            "Percent",
            "Count",
            width_ext = max_ext_width,
            width_bar = bar_width,
            width_tokens = max_tokens_width,
            width_count = max_count_width
        );

        let mut all_items = vec![
            ListItem::new(header).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            ListItem::new("─".repeat(available_width.min(120)))
                .style(Style::default().fg(Color::DarkGray)),
        ];
        all_items.extend(items);

        let extensions_widget = List::new(all_items)
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
