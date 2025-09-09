//! File selection widget for directory tree navigation and file selection.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// State for the file selection widget
#[derive(Debug, Clone)]
pub struct FileSelectionState {
    pub tree_cursor: usize,
    pub search_query: String,
    pub file_tree_scroll: u16,
}

impl FileSelectionState {
    pub fn from_model(model: &Model) -> Self {
        Self {
            tree_cursor: model.tree_cursor,
            search_query: model.search_query.clone(),
            file_tree_scroll: model.file_tree_scroll,
        }
    }
}

/// Widget for file selection with directory tree, search, and filter patterns
pub struct FileSelectionWidget<'a> {
    pub model: &'a Model,
}

impl<'a> FileSelectionWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Handle key events for file selection
    pub fn handle_key_event(key: KeyEvent, model: &Model, search_mode: bool) -> Option<Message> {
        // Handle search mode input
        if search_mode {
            match key.code {
                KeyCode::Esc => {
                    // Exit search mode
                    return Some(Message::ExitSearchMode);
                }
                KeyCode::Char(c) => {
                    let mut query = model.search_query.clone();
                    query.push(c);
                    return Some(Message::UpdateSearchQuery(query));
                }
                KeyCode::Backspace => {
                    let mut query = model.search_query.clone();
                    query.pop();
                    return Some(Message::UpdateSearchQuery(query));
                }
                KeyCode::Enter => {
                    // Exit search mode and run analysis
                    return Some(Message::ExitSearchMode);
                }
                _ => return None,
            }
        }

        // Handle normal mode input
        match key.code {
            KeyCode::Up => Some(Message::MoveTreeCursor(-1)),
            KeyCode::Down => Some(Message::MoveTreeCursor(1)),
            KeyCode::Char(' ') => Some(Message::ToggleFileSelection(model.tree_cursor)),
            KeyCode::Enter => Some(Message::RunAnalysis),
            KeyCode::Right => {
                let visible_nodes = model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(model.tree_cursor) {
                    if node.is_directory && !node.is_expanded {
                        Some(Message::ExpandDirectory(model.tree_cursor))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            KeyCode::Left => {
                let visible_nodes = model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(model.tree_cursor) {
                    if node.is_directory && node.is_expanded {
                        Some(Message::CollapseDirectory(model.tree_cursor))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Enter search mode
                Some(Message::EnterSearchMode)
            }
            KeyCode::PageUp => Some(Message::ScrollFileTree(-5)),
            KeyCode::PageDown => Some(Message::ScrollFileTree(5)),
            KeyCode::Home => Some(Message::ScrollFileTree(-9999)), // Scroll to top
            KeyCode::End => Some(Message::ScrollFileTree(9999)),   // Scroll to bottom
            _ => None,
        }
    }
}

impl<'a> StatefulWidget for FileSelectionWidget<'a> {
    type State = FileSelectionState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // File tree
                Constraint::Length(3), // Search bar
                Constraint::Length(3), // Pattern info
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // File tree with scroll support
        let visible_nodes = self.model.get_visible_nodes();
        let total_nodes = visible_nodes.len();

        // Calculate viewport dimensions
        let tree_area = layout[0];
        let content_height = tree_area.height.saturating_sub(2) as usize; // Account for borders

        // Calculate scroll position and viewport
        let scroll_start = state.file_tree_scroll as usize;
        let scroll_end = (scroll_start + content_height).min(total_nodes);

        // Create items only for visible viewport
        let items: Vec<ListItem> = visible_nodes
            .iter()
            .enumerate()
            .skip(scroll_start)
            .take(content_height)
            .map(|(i, node)| {
                let indent = "  ".repeat(node.level);
                let icon = if node.is_directory {
                    if node.is_expanded {
                        "📂"
                    } else {
                        "📁"
                    }
                } else {
                    "📄"
                };
                let checkbox = if node.is_selected { "☑" } else { "☐" };

                let content = format!("{}{} {} {}", indent, icon, checkbox, node.name);
                let mut style = Style::default();

                // Adjust cursor position for viewport
                if i == state.tree_cursor {
                    style = style.bg(Color::Blue).fg(Color::White);
                }

                if node.is_selected {
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

        // Search bar (moved below tree) with red 'S' to indicate hotkey
        // Create a styled title with red 'S'
        let title_spans = vec![
            Span::styled(
                "s",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("earch", Style::default().fg(Color::White)),
            Span::styled(
                if state.search_query.contains('*') {
                    " (glob pattern active)"
                } else {
                    " (text or glob pattern)"
                },
                Style::default().fg(Color::Gray),
            ),
        ];

        let search_widget = Paragraph::new(state.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans)),
            )
            .style(Style::default().fg(if state.search_query.contains('*') {
                Color::Yellow
            } else {
                Color::Green
            }));
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
            "↑↓: Navigate | Space: Select/Deselect | ←→: Expand/Collapse | PgUp/PgDn: Scroll | Enter: Run Analysis | S: Search Mode | Esc: Exit"
        )
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
        Widget::render(instructions, layout[3], buf);
    }
}
