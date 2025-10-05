//! File selection widget for directory tree navigation and file selection.

use crate::model::Model;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// State for the file selection widget - no longer needed, read directly from Model
pub type FileSelectionState = ();

/// Widget for file selection with directory tree, search, and filter patterns
pub struct FileSelectionWidget<'a> {
    pub model: &'a Model,
}

impl<'a> FileSelectionWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl<'a> StatefulWidget for FileSelectionWidget<'a> {
    type State = FileSelectionState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // File tree
                Constraint::Length(3), // Search bar
                Constraint::Length(3), // Pattern info
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // File tree with scroll support - use new session-based approach
        let mut session_clone = self.model.session.clone();
        let visible_nodes = crate::utils::get_visible_nodes(
            &self.model.file_tree_nodes,
            &self.model.search_query,
            &mut session_clone,
        );
        let total_nodes = visible_nodes.len();

        // Calculate viewport dimensions
        let tree_area = layout[0];
        let content_height = tree_area.height.saturating_sub(2) as usize; // Account for borders

        // Calculate scroll position and viewport
        let scroll_start = self.model.file_tree_scroll as usize;
        let scroll_end = (scroll_start + content_height).min(total_nodes);

        // Create items only for visible viewport
        let items: Vec<ListItem> = visible_nodes
            .iter()
            .enumerate()
            .skip(scroll_start)
            .take(content_height)
            .map(|(i, display_node)| {
                let node = &display_node.node;
                let is_selected = display_node.is_selected;

                let indent = "  ".repeat(node.level);
                let icon = if node.is_directory {
                    if node.is_expanded { "📂" } else { "📁" }
                } else {
                    "📄"
                };
                let checkbox = if is_selected { "☑" } else { "☐" };

                let content = format!("{}{} {} {}", indent, icon, checkbox, node.name);
                let mut style = Style::default();

                // Adjust cursor position for viewport
                if i == self.model.tree_cursor {
                    style = style.bg(Color::Blue).fg(Color::White);
                }

                if is_selected {
                    style = style.fg(Color::Green);
                }

                ListItem::new(content).style(style)
            })
            .collect();

        // Create title with scroll indicator
        let scroll_indicator = if total_nodes > content_height {
            let current_start = scroll_start + 1;
            let current_end = scroll_end;
            format!(
                "Files ({}) | Showing {}-{} of {}",
                total_nodes, current_start, current_end, total_nodes
            )
        } else {
            format!("Files ({})", total_nodes)
        };

        let tree_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(scroll_indicator),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        Widget::render(tree_widget, layout[0], buf);

        // Search bar - read directly from Model
        let title_spans = vec![
            Span::styled(
                "s",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("earch", Style::default().fg(Color::White)),
            Span::styled(" (text or * ? wildcards)", Style::default().fg(Color::Gray)),
        ];

        let search_widget = Paragraph::new(self.model.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans)),
            )
            .style(
                Style::default().fg(if self.model.search_query.contains('*') {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            );
        Widget::render(search_widget, layout[1], buf);

        // Pattern info
        let include_text = if self.model.session.config.include_patterns.is_empty() {
            "All files".to_string()
        } else {
            format!(
                "Include: {}",
                self.model.session.config.include_patterns.join(", ")
            )
        };
        let exclude_text = if self.model.session.config.exclude_patterns.is_empty() {
            "".to_string()
        } else {
            format!(
                " | Exclude: {}",
                self.model.session.config.exclude_patterns.join(", ")
            )
        };
        let pattern_info = format!("{}{}", include_text, exclude_text);

        let pattern_widget = Paragraph::new(pattern_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Filter Patterns"),
            )
            .style(Style::default().fg(Color::Cyan));
        Widget::render(pattern_widget, layout[2], buf);

        // Instructions
        let instructions = Paragraph::new(
            "Enter: Run Analysis | ↑↓: Navigate | Space: Select/Deselect | ←→: Expand/Collapse | PgUp/PgDn: Scroll | S: Search Mode | Esc: Exit"
        )
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
        Widget::render(instructions, layout[3], buf);
    }
}
