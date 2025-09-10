//! Statistics overview widget for displaying analysis summary.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// Widget for statistics overview (stateless)
pub struct StatisticsOverviewWidget<'a> {
    pub model: &'a Model,
}

impl<'a> StatisticsOverviewWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Handle key events for statistics overview
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

    /// Count selected files in the tree
    fn count_selected_files(nodes: &[crate::model::FileNode]) -> usize {
        let mut count = 0;
        for node in nodes {
            if node.is_selected && !node.is_directory {
                count += 1;
            }
            count += Self::count_selected_files(&node.children);
        }
        count
    }

    /// Count total files in the tree
    fn count_total_files(nodes: &[crate::model::FileNode]) -> usize {
        let mut count = 0;
        for node in nodes {
            if !node.is_directory {
                count += 1;
            }
            count += Self::count_total_files(&node.children);
        }
        count
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

impl<'a> Widget for StatisticsOverviewWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Statistics content
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Check if analysis has been run
        if self.model.prompt_output.generated_prompt.is_none()
            && !self.model.prompt_output.analysis_in_progress
        {
            // Show placeholder when no analysis has been run
            let placeholder_text = "📊 Statistics & Analysis\n\nNo analysis data available yet.\n\nTo view statistics:\n1. Go to Selection tab (Tab/Shift+Tab)\n2. Select files to analyze\n3. Press Enter to run analysis\n4. Return here to view results\n\nPress Enter to run analysis.";

            let placeholder_widget = Paragraph::new(placeholder_text)
                .block(Block::default().borders(Borders::ALL).title("📊 Overview"))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            Widget::render(placeholder_widget, layout[0], buf);

            // Instructions for when no analysis is available
            let instructions = Paragraph::new("Enter: Go to Selection | Tab/Shift+Tab: Switch Tab")
                .block(Block::default().borders(Borders::ALL).title("Controls"))
                .style(Style::default().fg(Color::Gray));
            Widget::render(instructions, layout[1], buf);
            return;
        }

        let mut stats_items: Vec<ListItem> = Vec::new();

        // Analysis Status (most important first)
        let (status_text, status_color) = if self.model.prompt_output.analysis_in_progress {
            ("Generating prompt...".to_string(), Color::Yellow)
        } else if self.model.prompt_output.analysis_error.is_some() {
            ("Analysis failed".to_string(), Color::Red)
        } else if self.model.prompt_output.generated_prompt.is_some() {
            ("Analysis complete".to_string(), Color::Green)
        } else {
            ("Ready to analyze".to_string(), Color::Gray)
        };

        stats_items.push(
            ListItem::new(format!("Status: {}", status_text)).style(
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        );

        if let Some(error) = &self.model.prompt_output.analysis_error {
            stats_items.push(
                ListItem::new(format!("  Error: {}", error)).style(Style::default().fg(Color::Red)),
            );
        }
        stats_items.push(ListItem::new(""));

        // File Summary
        stats_items.push(
            ListItem::new("📁 File Summary").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        );

        let selected_count = Self::count_selected_files(&self.model.file_tree.file_tree);
        let eligible_count = Self::count_total_files(&self.model.file_tree.file_tree);
        let total_files = self.model.prompt_output.file_count;
        stats_items.push(ListItem::new(format!(
            "  • Selected: {} files",
            selected_count
        )));
        stats_items.push(ListItem::new(format!(
            "  • Eligible: {} files",
            eligible_count
        )));
        stats_items.push(ListItem::new(format!(
            "  • Total Found: {} files",
            total_files
        )));

        if selected_count > 0 && eligible_count > 0 {
            let percentage = (selected_count as f64 / eligible_count as f64 * 100.0) as usize;
            stats_items.push(ListItem::new(format!(
                "  • Selection Rate: {}%",
                percentage
            )));
        }
        stats_items.push(ListItem::new(""));

        // Token Summary
        stats_items.push(
            ListItem::new("🎯 Token Summary").style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        );

        if let Some(token_count) = self.model.prompt_output.token_count {
            stats_items.push(ListItem::new(format!(
                "  • Total Tokens: {}",
                Self::format_number(token_count, &self.model.session.session.config.token_format)
            )));
            if selected_count > 0 {
                let avg_tokens = token_count / selected_count;
                stats_items.push(ListItem::new(format!(
                    "  • Avg per File: {}",
                    Self::format_number(
                        avg_tokens,
                        &self.model.session.session.config.token_format
                    )
                )));
            }
        } else {
            stats_items.push(ListItem::new("  • Total Tokens: Not calculated"));
        }
        stats_items.push(ListItem::new(""));

        // Configuration Summary
        stats_items.push(
            ListItem::new("⚙️  Configuration").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );

        let output_format = match self.model.session.session.config.output_format {
            code2prompt_core::template::OutputFormat::Markdown => "Markdown",
            code2prompt_core::template::OutputFormat::Json => "JSON",
            code2prompt_core::template::OutputFormat::Xml => "XML",
        };
        stats_items.push(ListItem::new(format!("  • Output: {}", output_format)));
        stats_items.push(ListItem::new(format!(
            "  • Line Numbers: {}",
            if self.model.session.session.config.line_numbers {
                "On"
            } else {
                "Off"
            }
        )));
        stats_items.push(ListItem::new(format!(
            "  • Git Diff: {}",
            if self.model.session.session.config.diff_enabled {
                "On"
            } else {
                "Off"
            }
        )));

        let pattern_summary = format!(
            "  • Patterns: {} include, {} exclude",
            self.model.session.session.config.include_patterns.len(),
            self.model.session.session.config.exclude_patterns.len()
        );
        stats_items.push(ListItem::new(pattern_summary));

        let stats_widget = List::new(stats_items)
            .block(Block::default().borders(Borders::ALL).title("📊 Overview"))
            .style(Style::default().fg(Color::White));

        Widget::render(stats_widget, layout[0], buf);

        // Instructions
        let instructions =
            Paragraph::new("Enter: Run Analysis | ←→: Switch View | Tab/Shift+Tab: Switch Tab")
                .block(Block::default().borders(Borders::ALL).title("Controls"))
                .style(Style::default().fg(Color::Gray));
        Widget::render(instructions, layout[1], buf);
    }
}
