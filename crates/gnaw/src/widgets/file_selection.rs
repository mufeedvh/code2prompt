//! File selection widget for directory tree navigation and file selection.

use crate::model::DisplayFileNode;
use crate::model::Model;
use crate::model::TokenState;
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

/// Braille snake spinner (npm/pnpm style). Frame chosen from wall-clock time so it
/// animates across the TUI's redraw loop without any stored tick counter.
fn spinner_frame() -> char {
    const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    const INTERVAL_MS: u128 = 80; // ~12.5 fps, same cadence as npm
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    FRAMES[((ms / INTERVAL_MS) % FRAMES.len() as u128) as usize]
}

impl<'a> FileSelectionWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

/// True if any loaded descendant is Pending or Counting.
fn dir_has_pending(node: &DisplayFileNode, model: &crate::model::Model) -> bool {
    if node.is_directory {
        node.children.iter().any(|c| dir_has_pending(c, model))
    } else {
        matches!(
            model.token_states.get(&node.path),
            Some(TokenState::Pending) | Some(TokenState::Counting)
        )
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
            self.model.size_filter,
            &self.model.token_states,
            &mut session_clone,
        );
        let total_nodes = visible_nodes.len();

        // Calculate viewport dimensions
        let tree_area = layout[0];
        let content_height = tree_area.height.saturating_sub(2).max(1) as usize; // Account for borders, keep >= 1

        // Derive a local, clamped scroll that keeps the cursor visible
        let cursor = self.model.tree_cursor.min(total_nodes.saturating_sub(1));
        let mut scroll_start = self.model.file_tree_scroll as usize;
        if cursor < scroll_start {
            scroll_start = cursor;
        } else if cursor >= scroll_start.saturating_add(content_height) {
            scroll_start = cursor.saturating_add(1).saturating_sub(content_height);
        }
        let max_scroll = total_nodes.saturating_sub(content_height);
        scroll_start = scroll_start.min(max_scroll);

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

                let token_suffix = if node.is_directory {
                    // While descendants are still counting, the stored agg is stale →
                    // show the spinner. Once quiescent, agg_tokens is fresh and correct.
                    if dir_has_pending(node, self.model) {
                        format!("  [{}]", spinner_frame())
                    } else {
                        let total = node.agg_tokens.unwrap_or(0);
                        if total > 0 {
                            let count = crate::utils::format_number(
                                total,
                                &self.model.session.config.token_format,
                            );
                            if self.model.selected_token_total > 0 {
                                let pct =
                                    total as f64 / self.model.selected_token_total as f64 * 100.0;
                                if pct >= 0.1 {
                                    format!("  [{} · {:.1}%]", count, pct)
                                } else {
                                    format!("  [{} · <0.1%]", count)
                                }
                            } else {
                                format!("  [{}]", count)
                            }
                        } else {
                            String::new()
                        }
                    }
                } else if is_selected {
                    match self.model.token_states.get(&node.path) {
                        Some(TokenState::Done(n)) => {
                            let count = crate::utils::format_number(
                                *n,
                                &self.model.session.config.token_format,
                            );
                            if self.model.selected_token_total > 0 {
                                let pct =
                                    *n as f64 / self.model.selected_token_total as f64 * 100.0;
                                // floor sub-0.1% to avoid a misleading "0.0%"
                                if pct >= 0.1 {
                                    format!("  [{} · {:.1}%]", count, pct)
                                } else {
                                    format!("  [{} · <0.1%]", count)
                                }
                            } else {
                                format!("  [{}]", count)
                            }
                        }
                        Some(TokenState::Counting) => {
                            format!("  [{}]", spinner_frame())
                        }
                        Some(TokenState::Pending) => "  [·]".to_string(),
                        Some(TokenState::Failed) => "  [—]".to_string(),
                        None => String::new(),
                    }
                } else {
                    String::new() // unselected rows carry no prompt share
                };

                let content = format!(
                    "{}{} {} {}{}",
                    indent, icon, checkbox, node.name, token_suffix
                );
                let mut style = Style::default();

                // Adjust cursor position for viewport
                if i == cursor {
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
