//! File selection widget for directory tree navigation and file selection.

use crate::model::{FileNode, Message, Model};
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
            tree_cursor: model.file_tree.tree_cursor,
            search_query: model.file_tree.search_query.clone(),
            file_tree_scroll: model.file_tree.file_tree_scroll,
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
                    let mut query = model.file_tree.search_query.clone();
                    query.push(c);
                    return Some(Message::UpdateSearchQuery(query));
                }
                KeyCode::Backspace => {
                    let mut query = model.file_tree.search_query.clone();
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
            KeyCode::Char(' ') => Some(Message::ToggleFileSelection(model.file_tree.tree_cursor)),
            KeyCode::Enter => Some(Message::RunAnalysis),
            KeyCode::Right => {
                let visible_nodes = model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(model.file_tree.tree_cursor) {
                    if node.is_directory && !node.is_expanded {
                        Some(Message::ExpandDirectory(model.file_tree.tree_cursor))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            KeyCode::Left => {
                let visible_nodes = model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(model.file_tree.tree_cursor) {
                    if node.is_directory && node.is_expanded {
                        Some(Message::CollapseDirectory(model.file_tree.tree_cursor))
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

    /// Toggle directory selection and all its children
    pub fn toggle_directory_selection(file_tree: &mut [FileNode], path: &str, selected: bool) {
        // Update the directory itself in the tree
        Self::update_node_selection_recursive(file_tree, path, selected);

        // Recursively update all children
        Self::toggle_directory_children_selection(file_tree, path, selected);
    }

    fn toggle_directory_children_selection(nodes: &mut [FileNode], dir_path: &str, selected: bool) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == dir_path && node.is_directory {
                // Found the directory, now update all its children recursively
                Self::select_all_children(&mut node.children, selected);
                return;
            }
            Self::toggle_directory_children_selection(&mut node.children, dir_path, selected);
        }
    }

    fn select_all_children(nodes: &mut [FileNode], selected: bool) {
        for node in nodes.iter_mut() {
            node.is_selected = selected;
            // Recursively select children if this is a directory
            if node.is_directory {
                Self::select_all_children(&mut node.children, selected);
            }
        }
    }

    /// Update selection state of a specific node
    pub fn update_node_selection(file_tree: &mut [FileNode], path: &str, selected: bool) {
        Self::update_node_selection_recursive(file_tree, path, selected);
    }

    fn update_node_selection_recursive(nodes: &mut [FileNode], path: &str, selected: bool) -> bool {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == path {
                node.is_selected = selected;
                return true;
            }
            if Self::update_node_selection_recursive(&mut node.children, path, selected) {
                return true;
            }
        }
        false
    }

    /// Expand a directory in the file tree
    pub fn expand_directory(file_tree: &mut [FileNode], path: &str) {
        Self::expand_directory_recursive(file_tree, path);
    }

    fn expand_directory_recursive(nodes: &mut [FileNode], path: &str) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == path && node.is_directory {
                node.is_expanded = true;
                // Load children if not already loaded - simplified approach
                if node.children.is_empty() {
                    if let Ok(entries) = std::fs::read_dir(&node.path) {
                        for entry in entries.flatten() {
                            let child_path = entry.path();
                            let mut child_node = FileNode::new(child_path, node.level + 1);
                            child_node.is_selected = false; // New directories are not selected by default
                            node.children.push(child_node);
                        }
                        // Sort children
                        node.children
                            .sort_by(|a, b| match (a.is_directory, b.is_directory) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.name.cmp(&b.name),
                            });
                    }
                }
                return;
            }
            Self::expand_directory_recursive(&mut node.children, path);
        }
    }

    /// Collapse a directory in the file tree
    pub fn collapse_directory(file_tree: &mut [FileNode], path: &str) {
        Self::collapse_directory_recursive(file_tree, path);
    }

    fn collapse_directory_recursive(nodes: &mut [FileNode], target_path: &str) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == target_path && node.is_directory {
                node.is_expanded = false;
                return;
            }
            Self::collapse_directory_recursive(&mut node.children, target_path);
        }
    }

    /// Adjust file tree scroll to keep the cursor visible in the viewport
    pub fn adjust_file_tree_scroll_for_cursor(
        tree_cursor: usize,
        file_tree_scroll: &mut u16,
        visible_count: usize,
    ) {
        if visible_count == 0 {
            return;
        }

        // Estimate viewport height (this will be more accurate in practice)
        let viewport_height = 20; // This should match the actual content height in render

        let cursor_pos = tree_cursor;
        let scroll_pos = *file_tree_scroll as usize;

        // If cursor is above viewport, scroll up
        if cursor_pos < scroll_pos {
            *file_tree_scroll = cursor_pos as u16;
        }
        // If cursor is below viewport, scroll down
        else if cursor_pos >= scroll_pos + viewport_height {
            *file_tree_scroll = (cursor_pos.saturating_sub(viewport_height - 1)) as u16;
        }

        // Ensure scroll doesn't go beyond bounds
        let max_scroll = visible_count.saturating_sub(viewport_height);
        if *file_tree_scroll as usize > max_scroll {
            *file_tree_scroll = max_scroll as u16;
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
        let visible_nodes = self.model.file_tree.get_visible_nodes();
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
        let include_text = if self
            .model
            .session
            .session
            .config
            .include_patterns
            .is_empty()
        {
            "All files".to_string()
        } else {
            format!(
                "Include: {}",
                self.model
                    .session
                    .session
                    .config
                    .include_patterns
                    .join(", ")
            )
        };
        let exclude_text = if self
            .model
            .session
            .session
            .config
            .exclude_patterns
            .is_empty()
        {
            "".to_string()
        } else {
            format!(
                " | Exclude: {}",
                self.model
                    .session
                    .session
                    .config
                    .exclude_patterns
                    .join(", ")
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
