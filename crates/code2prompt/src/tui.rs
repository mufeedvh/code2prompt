use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io::{stdout, Stdout};
use tokio::sync::mpsc;

use crate::model::{Message, Model, SettingAction, Tab};
use crate::utils::{build_file_tree, run_analysis};

pub struct TuiApp {
    model: Model,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    message_tx: mpsc::UnboundedSender<Message>,
    message_rx: mpsc::UnboundedReceiver<Message>,
}

impl TuiApp {
    pub fn new_with_args(
        path: std::path::PathBuf,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    ) -> Result<Self> {
        let terminal = init_terminal()?;
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let model = Model::new_with_cli_args(path, include_patterns, exclude_patterns);

        Ok(Self {
            model,
            terminal,
            message_tx,
            message_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Initialize file tree
        self.handle_message(Message::RefreshFileTree)?;

        loop {
            // Draw UI
            let model = self.model.clone();
            self.terminal.draw(|frame| {
                TuiApp::render_with_model(&model, frame);
            })?;

            // Handle events with timeout
            tokio::select! {
                // Handle keyboard events
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {
                    if event::poll(std::time::Duration::from_millis(0))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                if let Some(message) = self.handle_key_event(key) {
                                    self.handle_message(message)?;
                                }
                            }
                        }
                    }
                }

                // Handle internal messages
                Some(message) = self.message_rx.recv() => {
                    self.handle_message(message)?;
                }
            }

            if self.model.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn render_with_model(model: &Model, frame: &mut Frame) {
        let area = frame.area();

        // Main layout: tabs + content + status
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab bar
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status bar
            ])
            .split(area);

        // Render tab bar
        Self::render_tab_bar_static(model, frame, main_layout[0]);

        // Render current tab content
        match model.current_tab {
            Tab::FileTree => Self::render_file_tree_tab_static(model, frame, main_layout[1]),
            Tab::Settings => Self::render_settings_tab_static(model, frame, main_layout[1]),
            Tab::Statistics => Self::render_statistics_tab_static(model, frame, main_layout[1]),
            Tab::PromptOutput => {
                Self::render_prompt_output_tab_static(model, frame, main_layout[1])
            }
        }

        // Render status bar
        Self::render_status_bar_static(model, frame, main_layout[2]);
    }

    fn handle_key_event(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Global shortcuts
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(Message::Quit);
            }
            KeyCode::Esc => return Some(Message::Quit),
            KeyCode::Char('1') => return Some(Message::SwitchTab(Tab::FileTree)),
            KeyCode::Char('2') => return Some(Message::SwitchTab(Tab::Settings)),
            KeyCode::Char('3') => return Some(Message::SwitchTab(Tab::Statistics)),
            KeyCode::Char('4') => return Some(Message::SwitchTab(Tab::PromptOutput)),
            KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Cycle through tabs: Selection -> Settings -> Statistics -> Output -> Selection
                let next_tab = match self.model.current_tab {
                    Tab::FileTree => Tab::Settings,
                    Tab::Settings => Tab::Statistics,
                    Tab::Statistics => Tab::PromptOutput,
                    Tab::PromptOutput => Tab::FileTree,
                };
                return Some(Message::SwitchTab(next_tab));
            }
            KeyCode::BackTab | KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Cycle through tabs in reverse: Selection <- Settings <- Statistics <- Output <- Selection
                let prev_tab = match self.model.current_tab {
                    Tab::FileTree => Tab::PromptOutput,
                    Tab::Settings => Tab::FileTree,
                    Tab::Statistics => Tab::Settings,
                    Tab::PromptOutput => Tab::Statistics,
                };
                return Some(Message::SwitchTab(prev_tab));
            }
            _ => {}
        }

        // Tab-specific shortcuts
        match self.model.current_tab {
            Tab::FileTree => self.handle_file_tree_keys(key),
            Tab::Settings => self.handle_settings_keys(key),
            Tab::Statistics => self.handle_statistics_keys(key),
            Tab::PromptOutput => self.handle_prompt_output_keys(key),
        }
    }

    fn handle_file_tree_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Up => Some(Message::MoveTreeCursor(-1)),
            KeyCode::Down => Some(Message::MoveTreeCursor(1)),
            KeyCode::Char(' ') => Some(Message::ToggleFileSelection(self.model.tree_cursor)),
            KeyCode::Enter => Some(Message::RunAnalysis),
            KeyCode::Right => {
                let visible_nodes = self.model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(self.model.tree_cursor) {
                    if node.is_directory && !node.is_expanded {
                        Some(Message::ExpandDirectory(self.model.tree_cursor))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            KeyCode::Left => {
                let visible_nodes = self.model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(self.model.tree_cursor) {
                    if node.is_directory && node.is_expanded {
                        Some(Message::CollapseDirectory(self.model.tree_cursor))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' => {
                let mut query = self.model.search_query.clone();
                query.push(c);
                Some(Message::UpdateSearchQuery(query))
            }
            KeyCode::Backspace => {
                let mut query = self.model.search_query.clone();
                query.pop();
                Some(Message::UpdateSearchQuery(query))
            }
            KeyCode::Esc => {
                if !self.model.search_query.is_empty() {
                    Some(Message::UpdateSearchQuery(String::new()))
                } else {
                    None
                }
            }
            KeyCode::PageUp => Some(Message::ScrollFileTree(-5)),
            KeyCode::PageDown => Some(Message::ScrollFileTree(5)),
            KeyCode::Home => Some(Message::ScrollFileTree(-9999)), // Scroll to top
            KeyCode::End => Some(Message::ScrollFileTree(9999)),   // Scroll to bottom
            _ => None,
        }
    }

    fn handle_settings_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Up => Some(Message::MoveSettingsCursor(-1)),
            KeyCode::Down => Some(Message::MoveSettingsCursor(1)),
            KeyCode::Char(' ') => Some(Message::ToggleSetting(self.model.settings_cursor)),
            KeyCode::Left | KeyCode::Right => {
                Some(Message::CycleSetting(self.model.settings_cursor))
            }
            KeyCode::Enter => Some(Message::RunAnalysis),
            _ => None,
        }
    }
    
    fn handle_statistics_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => {
                // If no analysis has been run, switch to Selection tab
                if self.model.generated_prompt.is_none() && !self.model.analysis_in_progress {
                    Some(Message::SwitchTab(Tab::FileTree))
                } else {
                    Some(Message::RunAnalysis)
                }
            },
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

    fn handle_prompt_output_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => Some(Message::RunAnalysis),
            // Check for Ctrl+Up/Down first for faster scrolling
            KeyCode::Up
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                Some(Message::ScrollOutput(-10))
            }
            KeyCode::Down
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                Some(Message::ScrollOutput(10))
            }
            KeyCode::Up => Some(Message::ScrollOutput(-1)),
            KeyCode::Down => Some(Message::ScrollOutput(1)),
            KeyCode::PageUp => Some(Message::ScrollOutput(-5)),
            KeyCode::PageDown => Some(Message::ScrollOutput(5)),
            KeyCode::Home => Some(Message::ScrollOutput(-9999)), // Scroll to top
            KeyCode::End => Some(Message::ScrollOutput(9999)),   // Scroll to bottom
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.model.generated_prompt.is_some() {
                    Some(Message::CopyToClipboard)
                } else {
                    None
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.model.generated_prompt.is_some() {
                    Some(Message::SaveToFile("prompt.md".to_string()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn handle_message(&mut self, message: Message) -> Result<()> {
        match message {
            Message::Quit => {
                self.model.should_quit = true;
            }
            Message::SwitchTab(tab) => {
                self.model.current_tab = tab;
                self.model.status_message = format!("Switched to {:?} tab", tab);
            }
            Message::RefreshFileTree => {
                // Build file tree from config path
                match build_file_tree(&self.model.config) {
                    Ok(tree) => {
                        self.model.file_tree = tree;

                        // Sync selected_files HashMap with the tree state
                        self.model.selected_files.clear();
                        Self::sync_selected_files_from_tree(
                            &self.model.file_tree,
                            &mut self.model.selected_files,
                        );

                        self.model.status_message = "File tree refreshed".to_string();
                    }
                    Err(e) => {
                        self.model.status_message = format!("Error loading files: {}", e);
                    }
                }
            }
            Message::UpdateSearchQuery(query) => {
                self.model.search_query = query;
                // Reset cursor when search changes
                self.model.tree_cursor = 0;
            }
            Message::MoveTreeCursor(delta) => {
                let visible_count = self.model.get_visible_nodes().len();
                if visible_count > 0 {
                    let new_cursor = if delta > 0 {
                        (self.model.tree_cursor + delta as usize).min(visible_count - 1)
                    } else {
                        self.model.tree_cursor.saturating_sub((-delta) as usize)
                    };
                    self.model.tree_cursor = new_cursor;
                    
                    // Auto-adjust scroll to keep cursor visible
                    self.adjust_file_tree_scroll_for_cursor();
                }
            }
            Message::MoveSettingsCursor(delta) => {
                let settings_count = self.model.get_settings_items().len();
                if settings_count > 0 {
                    let new_cursor = if delta > 0 {
                        (self.model.settings_cursor + delta as usize).min(settings_count - 1)
                    } else {
                        self.model.settings_cursor.saturating_sub((-delta) as usize)
                    };
                    self.model.settings_cursor = new_cursor;
                }
            }
            Message::ToggleFileSelection(index) => {
                let visible_nodes = self.model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    let path = node.path.to_string_lossy().to_string();
                    let name = node.name.clone();
                    let is_directory = node.is_directory;
                    let current = self
                        .model
                        .selected_files
                        .get(&path)
                        .copied()
                        .unwrap_or(node.is_selected);

                    // Generate glob pattern for this file/directory
                    let glob_pattern = self.model.path_to_glob_pattern(&node.path, is_directory);

                    // Update patterns dynamically
                    if !current {
                        // Selecting: add to include patterns if not present, remove from exclude
                        if !self.model.config.include_patterns.contains(&glob_pattern) {
                            self.model
                                .config
                                .include_patterns
                                .push(glob_pattern.clone());
                        }
                        self.model
                            .config
                            .exclude_patterns
                            .retain(|p| p != &glob_pattern);
                    } else {
                        // Deselecting: add to exclude patterns if not in include, or remove from include
                        if self.model.config.include_patterns.contains(&glob_pattern) {
                            self.model
                                .config
                                .include_patterns
                                .retain(|p| p != &glob_pattern);
                        } else {
                            if !self.model.config.exclude_patterns.contains(&glob_pattern) {
                                self.model
                                    .config
                                    .exclude_patterns
                                    .push(glob_pattern.clone());
                            }
                        }
                    }

                    // Update the node in the tree (and recursively if it's a directory)
                    if is_directory {
                        self.toggle_directory_selection(&path, !current);
                    } else {
                        self.model.selected_files.insert(path.clone(), !current);
                        self.update_node_selection(&path, !current);
                    }

                    let action = if current { "Deselected" } else { "Selected" };
                    let extra = if is_directory { " (and contents)" } else { "" };
                    self.model.status_message =
                        format!("{} {}{} (pattern: {})", action, name, extra, glob_pattern);
                }
            }
            Message::ExpandDirectory(index) => {
                let visible_nodes = self.model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    if node.is_directory {
                        let path = node.path.to_string_lossy().to_string();
                        let name = node.name.clone();
                        self.expand_directory(&path);
                        self.model.status_message = format!("Expanded {}", name);
                    }
                }
            }
            Message::CollapseDirectory(index) => {
                let visible_nodes = self.model.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    if node.is_directory {
                        let path = node.path.to_string_lossy().to_string();
                        let name = node.name.clone();
                        self.collapse_directory(&path);
                        self.model.status_message = format!("Collapsed {}", name);
                    }
                }
            }
            Message::ToggleSetting(index) => {
                self.model.update_setting(index, SettingAction::Toggle);
                let settings = self.model.get_settings_items();
                if let Some(setting) = settings.get(index) {
                    self.model.status_message = format!("Toggled {}", setting.name);
                }
            }
            Message::CycleSetting(index) => {
                self.model.update_setting(index, SettingAction::Cycle);
                let settings = self.model.get_settings_items();
                if let Some(setting) = settings.get(index) {
                    self.model.status_message = format!("Cycled {}", setting.name);
                }
            }
            Message::RunAnalysis => {
                if !self.model.analysis_in_progress {
                    self.model.analysis_in_progress = true;
                    self.model.analysis_error = None;
                    self.model.status_message = "Running analysis...".to_string();

                    // Switch to prompt output tab
                    self.model.current_tab = Tab::PromptOutput;

                    // Run analysis in background
                    let config = self.model.config.clone();
                    let tx = self.message_tx.clone();

                    tokio::spawn(async move {
                        match run_analysis(config).await {
                            Ok(result) => {
                                let _ = tx.send(Message::AnalysisComplete(result));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::AnalysisError(e.to_string()));
                            }
                        }
                    });
                }
            }
            Message::AnalysisComplete(results) => {
                self.model.analysis_in_progress = false;
                self.model.generated_prompt = Some(results.generated_prompt);
                self.model.token_count = results.token_count;
                self.model.file_count = results.file_count;
                self.model.token_map_entries = results.token_map_entries;
                let tokens = results.token_count.unwrap_or(0);
                self.model.status_message =
                    format!("Analysis complete! {} tokens, {} files", tokens, results.file_count);
            }
            Message::AnalysisError(error) => {
                self.model.analysis_in_progress = false;
                self.model.analysis_error = Some(error.clone());
                self.model.status_message = format!("Analysis failed: {}", error);
            }
            Message::CopyToClipboard => {
                if let Some(prompt) = &self.model.generated_prompt {
                    match crate::utils::copy_to_clipboard(prompt) {
                        Ok(_) => {
                            self.model.status_message = "Copied to clipboard!".to_string();
                        }
                        Err(e) => {
                            self.model.status_message = format!("Copy failed: {}", e);
                        }
                    }
                }
            }
            Message::SaveToFile(filename) => {
                if let Some(prompt) = &self.model.generated_prompt {
                    match crate::utils::save_to_file(&filename, prompt) {
                        Ok(_) => {
                            self.model.status_message = format!("Saved to {}", filename);
                        }
                        Err(e) => {
                            self.model.status_message = format!("Save failed: {}", e);
                        }
                    }
                }
            }
            Message::ScrollOutput(delta) => {
                if let Some(prompt) = &self.model.generated_prompt {
                    // Calculate approximate total lines (rough estimate)
                    let total_lines = prompt.lines().count() as u16;
                    let viewport_height = 20; // Approximate viewport height
                    let max_scroll = total_lines.saturating_sub(viewport_height);

                    let new_scroll = if delta < 0 {
                        self.model.output_scroll.saturating_sub((-delta) as u16)
                    } else {
                        self.model.output_scroll.saturating_add(delta as u16)
                    };

                    // Clamp scroll to valid range
                    self.model.output_scroll = new_scroll.min(max_scroll);
                }
            }
            Message::ScrollFileTree(delta) => {
                let visible_count = self.model.get_visible_nodes().len() as u16;
                let viewport_height = 20; // Approximate viewport height for file tree
                let max_scroll = visible_count.saturating_sub(viewport_height);

                let new_scroll = if delta < 0 {
                    self.model.file_tree_scroll.saturating_sub((-delta) as u16)
                } else {
                    self.model.file_tree_scroll.saturating_add(delta as u16)
                };

                // Clamp scroll to valid range
                self.model.file_tree_scroll = new_scroll.min(max_scroll);
            }
            Message::CycleStatisticsView(direction) => {
                self.model.statistics_view = if direction > 0 {
                    // Next view (forward)
                    match self.model.statistics_view {
                        crate::model::StatisticsView::Overview => crate::model::StatisticsView::TokenMap,
                        crate::model::StatisticsView::TokenMap => crate::model::StatisticsView::Extensions,
                        crate::model::StatisticsView::Extensions => crate::model::StatisticsView::Overview,
                    }
                } else {
                    // Previous view (backward)
                    match self.model.statistics_view {
                        crate::model::StatisticsView::Overview => crate::model::StatisticsView::Extensions,
                        crate::model::StatisticsView::TokenMap => crate::model::StatisticsView::Overview,
                        crate::model::StatisticsView::Extensions => crate::model::StatisticsView::TokenMap,
                    }
                };
                self.model.statistics_scroll = 0; // Reset scroll when changing views
                let view_name = match self.model.statistics_view {
                    crate::model::StatisticsView::Overview => "Overview",
                    crate::model::StatisticsView::TokenMap => "Token Map",
                    crate::model::StatisticsView::Extensions => "Extensions",
                };
                self.model.status_message = format!("Switched to {} view", view_name);
            }
            Message::ScrollStatistics(delta) => {
                // For now, simple scroll logic - will be refined per view
                let new_scroll = if delta < 0 {
                    self.model.statistics_scroll.saturating_sub((-delta) as u16)
                } else {
                    self.model.statistics_scroll.saturating_add(delta as u16)
                };
                self.model.statistics_scroll = new_scroll;
            }
            Message::AddIncludePattern(pattern) => {
                if !self.model.config.include_patterns.contains(&pattern) {
                    self.model.config.include_patterns.push(pattern.clone());
                    self.model.status_message = format!("Added include pattern: {}", pattern);
                    // Refresh file tree to apply new pattern
                    self.handle_message(Message::RefreshFileTree)?;
                }
            }
            Message::AddExcludePattern(pattern) => {
                if !self.model.config.exclude_patterns.contains(&pattern) {
                    self.model.config.exclude_patterns.push(pattern.clone());
                    self.model.status_message = format!("Added exclude pattern: {}", pattern);
                    // Refresh file tree to apply new pattern
                    self.handle_message(Message::RefreshFileTree)?;
                }
            }
            Message::RemoveIncludePattern(pattern) => {
                self.model.config.include_patterns.retain(|p| p != &pattern);
                self.model.status_message = format!("Removed include pattern: {}", pattern);
                // Refresh file tree to apply new pattern
                self.handle_message(Message::RefreshFileTree)?;
            }
            Message::RemoveExcludePattern(pattern) => {
                self.model.config.exclude_patterns.retain(|p| p != &pattern);
                self.model.status_message = format!("Removed exclude pattern: {}", pattern);
                // Refresh file tree to apply new pattern
                self.handle_message(Message::RefreshFileTree)?;
            }
        }

        Ok(())
    }

    // Helper methods for tree manipulation
    fn sync_selected_files_from_tree(
        nodes: &[crate::model::FileNode],
        selected_files: &mut std::collections::HashMap<String, bool>,
    ) {
        for node in nodes {
            if node.is_selected {
                selected_files.insert(node.path.to_string_lossy().to_string(), true);
            }
            Self::sync_selected_files_from_tree(&node.children, selected_files);
        }
    }

    fn toggle_directory_selection(&mut self, path: &str, selected: bool) {
        // Update the directory itself
        self.model.selected_files.insert(path.to_string(), selected);
        Self::update_node_selection_recursive(&mut self.model.file_tree, path, selected);

        // Recursively update all children
        Self::toggle_directory_children_selection(
            &mut self.model.file_tree,
            path,
            selected,
            &mut self.model.selected_files,
        );
    }

    fn toggle_directory_children_selection(
        nodes: &mut [crate::model::FileNode],
        dir_path: &str,
        selected: bool,
        selected_files: &mut std::collections::HashMap<String, bool>,
    ) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == dir_path && node.is_directory {
                // Found the directory, now update all its children recursively
                Self::select_all_children(&mut node.children, selected, selected_files);
                return;
            }
            Self::toggle_directory_children_selection(
                &mut node.children,
                dir_path,
                selected,
                selected_files,
            );
        }
    }

    fn select_all_children(
        nodes: &mut [crate::model::FileNode],
        selected: bool,
        selected_files: &mut std::collections::HashMap<String, bool>,
    ) {
        for node in nodes.iter_mut() {
            node.is_selected = selected;
            let path = node.path.to_string_lossy().to_string();
            selected_files.insert(path, selected);

            // Recursively select children if this is a directory
            if node.is_directory {
                Self::select_all_children(&mut node.children, selected, selected_files);
            }
        }
    }

    fn update_node_selection(&mut self, path: &str, selected: bool) {
        Self::update_node_selection_recursive(&mut self.model.file_tree, path, selected);
    }

    fn update_node_selection_recursive(
        nodes: &mut [crate::model::FileNode],
        path: &str,
        selected: bool,
    ) -> bool {
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

    fn expand_directory(&mut self, path: &str) {
        let mut tree = self.model.file_tree.clone();
        self.expand_directory_recursive(&mut tree, path);
        self.model.file_tree = tree;
    }

    fn expand_directory_recursive(&self, nodes: &mut [crate::model::FileNode], path: &str) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == path && node.is_directory {
                node.is_expanded = true;
                // Load children if not already loaded
                if node.children.is_empty() {
                    if let Ok(children) = crate::utils::load_directory_children_with_config(
                        &node.path,
                        node.level + 1,
                        Some(&self.model.config),
                    ) {
                        node.children = children;
                    }
                }
                return;
            }
            self.expand_directory_recursive(&mut node.children, path);
        }
    }

    fn collapse_directory(&mut self, path: &str) {
        let mut tree = self.model.file_tree.clone();
        self.collapse_directory_recursive(&mut tree, path);
        self.model.file_tree = tree;
    }

    fn collapse_directory_recursive(&self, nodes: &mut [crate::model::FileNode], path: &str) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == path && node.is_directory {
                node.is_expanded = false;
                return;
            }
            self.collapse_directory_recursive(&mut node.children, path);
        }
    }

    /// Adjust file tree scroll to keep the cursor visible in the viewport
    fn adjust_file_tree_scroll_for_cursor(&mut self) {
        let visible_count = self.model.get_visible_nodes().len();
        if visible_count == 0 {
            return;
        }

        // Estimate viewport height (this will be more accurate in practice)
        let viewport_height = 20; // This should match the actual content height in render
        
        let cursor_pos = self.model.tree_cursor;
        let scroll_pos = self.model.file_tree_scroll as usize;
        
        // If cursor is above viewport, scroll up
        if cursor_pos < scroll_pos {
            self.model.file_tree_scroll = cursor_pos as u16;
        }
        // If cursor is below viewport, scroll down
        else if cursor_pos >= scroll_pos + viewport_height {
            self.model.file_tree_scroll = (cursor_pos.saturating_sub(viewport_height - 1)) as u16;
        }
        
        // Ensure scroll doesn't go beyond bounds
        let max_scroll = visible_count.saturating_sub(viewport_height);
        if self.model.file_tree_scroll as usize > max_scroll {
            self.model.file_tree_scroll = max_scroll as u16;
        }
    }

    fn render_tab_bar_static(model: &Model, frame: &mut Frame, area: Rect) {
        let tabs = vec!["1. Selection", "2. Settings", "3. Statistics", "4. Output"];
        let selected = match model.current_tab {
            Tab::FileTree => 0,
            Tab::Settings => 1,
            Tab::Statistics => 2,
            Tab::PromptOutput => 3,
        };

        let tabs_widget = Tabs::new(tabs)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Code2Prompt TUI"),
            )
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs_widget, area);
    }

    fn render_file_tree_tab_static(model: &Model, frame: &mut Frame, area: Rect) {
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
        let visible_nodes = model.get_visible_nodes();
        let total_nodes = visible_nodes.len();
        
        // Calculate viewport dimensions
        let tree_area = layout[0];
        let content_height = tree_area.height.saturating_sub(2) as usize; // Account for borders
        
        // Calculate scroll position and viewport
        let scroll_start = model.file_tree_scroll as usize;
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
                if i == model.tree_cursor {
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

        frame.render_widget(tree_widget, layout[0]);

        // Search bar (moved below tree)
        let search_title = if model.search_query.contains('*') {
            "Search (glob pattern active)"
        } else {
            "Search (text or glob pattern)"
        };
        let search_widget = Paragraph::new(model.search_query.as_str())
            .block(Block::default().borders(Borders::ALL).title(search_title))
            .style(Style::default().fg(if model.search_query.contains('*') {
                Color::Yellow
            } else {
                Color::Green
            }));
        frame.render_widget(search_widget, layout[1]);

        // Pattern info
        let include_text = if model.config.include_patterns.is_empty() {
            "All files".to_string()
        } else {
            format!("Include: {}", model.config.include_patterns.join(", "))
        };
        let exclude_text = if model.config.exclude_patterns.is_empty() {
            "".to_string()
        } else {
            format!(" | Exclude: {}", model.config.exclude_patterns.join(", "))
        };
        let pattern_info = format!("{}{}", include_text, exclude_text);

        let pattern_widget = Paragraph::new(pattern_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Filter Patterns"),
            )
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(pattern_widget, layout[2]);

        // Instructions
        let instructions = Paragraph::new(
            "↑↓: Navigate | Space: Select/Deselect | ←→: Expand/Collapse | PgUp/PgDn: Scroll | Enter: Run Analysis | Type to Search"
        )
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, layout[3]);
    }

    fn render_settings_tab_static(model: &Model, frame: &mut Frame, area: Rect) {
        let settings_groups = model.get_settings_groups();

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

                if item_index == model.settings_cursor {
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

        frame.render_widget(settings_widget, layout[0]);

        // Instructions
        let instructions = Paragraph::new(
            "↑↓: Navigate | Space: Toggle | ←→: Cycle Options | Enter: Run Analysis",
        )
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, layout[1]);
    }

    fn render_statistics_tab_static(model: &Model, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Statistics content
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Check if analysis has been run
        if model.generated_prompt.is_none() && !model.analysis_in_progress {
            // Show placeholder when no analysis has been run
            let placeholder_text = "📊 Statistics & Analysis\n\nNo analysis data available yet.\n\nTo view statistics:\n1. Go to Selection tab (Tab/Shift+Tab)\n2. Select files to analyze\n3. Press Enter to run analysis\n4. Return here to view results\n\nPress Enter to go to Selection tab or run analysis.";
            
            let placeholder_widget = Paragraph::new(placeholder_text)
                .block(Block::default().borders(Borders::ALL).title("Statistics & Analysis"))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            
            frame.render_widget(placeholder_widget, layout[0]);
            
            // Instructions for when no analysis is available
            let instructions = Paragraph::new("Enter: Go to Selection | Tab/Shift+Tab: Switch Tab")
                .block(Block::default().borders(Borders::ALL).title("Controls"))
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(instructions, layout[1]);
            return;
        }

        // Create title with current view indicator
        let view_name = match model.statistics_view {
            crate::model::StatisticsView::Overview => "📊 Overview",
            crate::model::StatisticsView::TokenMap => "🗂️  Token Map",
            crate::model::StatisticsView::Extensions => "📁 By Extension",
        };
        let title = format!("Statistics & Analysis - {}", view_name);

        // Render based on current view
        match model.statistics_view {
            crate::model::StatisticsView::Overview => {
                Self::render_overview_view(model, frame, layout[0], &title);
            }
            crate::model::StatisticsView::TokenMap => {
                Self::render_token_map_view(model, frame, layout[0], &title);
            }
            crate::model::StatisticsView::Extensions => {
                Self::render_extensions_view(model, frame, layout[0], &title);
            }
        }

        // Instructions
        let instructions = Paragraph::new("Enter: Run Analysis | ←→: Switch View | ↑↓/PgUp/PgDn: Scroll | Tab/Shift+Tab: Switch Tab")
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, layout[1]);
    }

    fn render_overview_view(model: &Model, frame: &mut Frame, area: Rect, title: &str) {
        let mut stats_items: Vec<ListItem> = Vec::new();

        // Analysis Status (most important first)
        let status = if model.analysis_in_progress {
            ("🔄 Analysis in Progress...", Color::Yellow)
        } else if model.analysis_error.is_some() {
            ("❌ Analysis Failed", Color::Red)
        } else if model.generated_prompt.is_some() {
            ("✅ Analysis Complete", Color::Green)
        } else {
            ("⏳ Ready to Analyze", Color::Gray)
        };
        
        stats_items.push(
            ListItem::new(format!("Status: {}", status.0))
                .style(Style::default().fg(status.1).add_modifier(Modifier::BOLD))
        );
        
        if let Some(error) = &model.analysis_error {
            stats_items.push(
                ListItem::new(format!("  Error: {}", error))
                    .style(Style::default().fg(Color::Red))
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
        
        let selected_count = model.selected_files.values().filter(|&&v| v).count();
        let eligible_count = model.selected_files.len(); // Total eligible files (matching patterns)
        let total_files = model.file_count;
        stats_items.push(ListItem::new(format!("  • Selected: {} files", selected_count)));
        stats_items.push(ListItem::new(format!("  • Eligible: {} files", eligible_count)));
        stats_items.push(ListItem::new(format!("  • Total Found: {} files", total_files)));
        
        if selected_count > 0 && eligible_count > 0 {
            let percentage = (selected_count as f64 / eligible_count as f64 * 100.0) as usize;
            stats_items.push(ListItem::new(format!("  • Selection Rate: {}%", percentage)));
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

        if let Some(token_count) = model.token_count {
            stats_items.push(ListItem::new(format!("  • Total Tokens: {}", Self::format_number(token_count, &model.config.token_format))));
            if selected_count > 0 {
                let avg_tokens = token_count / selected_count;
                stats_items.push(ListItem::new(format!("  • Avg per File: {}", Self::format_number(avg_tokens, &model.config.token_format))));
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

        let output_format = match model.config.output_format {
            code2prompt_core::template::OutputFormat::Markdown => "Markdown",
            code2prompt_core::template::OutputFormat::Json => "JSON",
            code2prompt_core::template::OutputFormat::Xml => "XML",
        };
        stats_items.push(ListItem::new(format!("  • Output: {}", output_format)));
        stats_items.push(ListItem::new(format!("  • Line Numbers: {}", if model.config.line_numbers { "On" } else { "Off" })));
        stats_items.push(ListItem::new(format!("  • Git Diff: {}", if model.config.diff_enabled { "On" } else { "Off" })));
        
        let pattern_summary = format!("  • Patterns: {} include, {} exclude", 
            model.config.include_patterns.len(), 
            model.config.exclude_patterns.len()
        );
        stats_items.push(ListItem::new(pattern_summary));

        let stats_widget = List::new(stats_items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White));

        frame.render_widget(stats_widget, area);
    }

    fn render_token_map_view(model: &Model, frame: &mut Frame, area: Rect, title: &str) {
        if model.token_map_entries.is_empty() {
            let placeholder_text = if model.generated_prompt.is_some() {
                "🗂️  Token Map View\n\nNo token map data available.\nMake sure token_map is enabled in configuration.\n\nPress Enter to re-run analysis."
            } else {
                "🗂️  Token Map View\n\nRun analysis first to see token distribution.\n\nPress Enter to analyze selected files."
            };

            let placeholder_widget = Paragraph::new(placeholder_text)
                .block(Block::default().borders(Borders::ALL).title(title))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Gray));

            frame.render_widget(placeholder_widget, area);
            return;
        }

        // Calculate viewport for scrolling
        let content_height = area.height.saturating_sub(2) as usize; // Account for borders
        let scroll_start = model.statistics_scroll as usize;
        let scroll_end = (scroll_start + content_height).min(model.token_map_entries.len());

        // Create list items for visible entries with proper tree structure
        let items: Vec<ListItem> = model.token_map_entries
            .iter()
            .skip(scroll_start)
            .take(content_height)
            .enumerate()
            .map(|(viewport_index, entry)| {
                let actual_index = scroll_start + viewport_index;
                
                // Build tree prefix with proper vertical lines
                let mut prefix = String::new();
                
                // Add vertical lines for parent levels
                for d in 0..entry.depth {
                    if d < entry.depth - 1 {
                        // Check if we need a vertical line at this depth
                        let needs_line = model.token_map_entries
                            .iter()
                            .skip(actual_index + 1)
                            .take_while(|next_entry| next_entry.depth > d)
                            .any(|next_entry| next_entry.depth == d + 1);
                        
                        if needs_line {
                            prefix.push_str("│ ");
                        } else {
                            prefix.push_str("  ");
                        }
                    } else {
                        if entry.is_last {
                            prefix.push_str("└─");
                        } else {
                            prefix.push_str("├─");
                        }
                    }
                }
                
                // Special handling for root level
                if entry.depth == 0 && actual_index == 0 && entry.name != "(other files)" {
                    prefix = "┌─".to_string();
                }
                
                // Check if has children
                let has_children = model.token_map_entries
                    .get(actual_index + 1)
                    .map(|next| next.depth > entry.depth)
                    .unwrap_or(false);
                
                // Add the connecting character
                if entry.depth > 0 || entry.name == "(other files)" {
                    if has_children {
                        prefix.push('┬');
                    } else {
                        prefix.push('─');
                    }
                } else if actual_index == 0 {
                    prefix.push('┴');
                }
                
                prefix.push(' ');

                // Create the visual bar
                let bar_width: usize = 20;
                let filled_chars = ((entry.percentage / 100.0) * bar_width as f64) as usize;
                let bar = format!("{}{}",
                    "█".repeat(filled_chars),
                    "░".repeat(bar_width.saturating_sub(filled_chars))
                );

                // Format the tokens with K/M suffix
                let tokens_str = Self::format_number(entry.tokens, &model.config.token_format);

                // Determine color based on entry type and size
                let color = if entry.metadata.is_dir {
                    Color::Cyan
                } else {
                    match entry.name.split('.').last().unwrap_or("") {
                        "rs" => Color::Yellow,
                        "md" => Color::Green,
                        "toml" | "json" => Color::Magenta,
                        _ => Color::White,
                    }
                };

                let content = format!(
                    "{}{} │{}│ {:>6} ({:>4.1}%)",
                    prefix, entry.name,
                    bar, tokens_str, entry.percentage
                );

                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        // Create title with scroll indicator
        let scroll_title = if model.token_map_entries.len() > content_height {
            format!("{} | Showing {}-{} of {}",
                title,
                scroll_start + 1,
                scroll_end,
                model.token_map_entries.len()
            )
        } else {
            title.to_string()
        };

        let token_map_widget = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(scroll_title));

        frame.render_widget(token_map_widget, area);
    }

    fn render_extensions_view(model: &Model, frame: &mut Frame, area: Rect, title: &str) {
        if model.token_map_entries.is_empty() {
            let placeholder_text = if model.generated_prompt.is_some() {
                "📁 Extensions View\n\nNo token map data available.\nMake sure token_map is enabled in configuration.\n\nPress Enter to re-run analysis."
            } else {
                "📁 Extensions View\n\nRun analysis first to see token breakdown by file extension.\n\nPress Enter to analyze selected files."
            };

            let placeholder_widget = Paragraph::new(placeholder_text)
                .block(Block::default().borders(Borders::ALL).title(title))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Gray));

            frame.render_widget(placeholder_widget, area);
            return;
        }

        // Aggregate tokens by file extension
        let mut extension_stats: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
        let total_tokens = model.token_count.unwrap_or(0);

        for entry in &model.token_map_entries {
            if !entry.metadata.is_dir {
                let extension = entry.name.split('.').last()
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
        let content_height = area.height.saturating_sub(2) as usize;
        let scroll_start = model.statistics_scroll as usize;
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
                let bar = format!("{}{}",
                    "█".repeat(filled_chars),
                    "░".repeat(bar_width.saturating_sub(filled_chars))
                );

                // Choose color based on extension
                let color = match extension.as_str() {
                    ".rs" => Color::Yellow,
                    ".md" => Color::Green,
                    ".toml" | ".json" | ".yaml" | ".yml" => Color::Magenta,
                    ".js" | ".ts" | ".jsx" | ".tsx" => Color::Cyan,
                    ".py" => Color::Blue,
                    ".go" => Color::LightBlue,
                    ".java" | ".kt" => Color::Red,
                    ".cpp" | ".c" | ".h" => Color::LightYellow,
                    _ => Color::White,
                };

                let content = format!(
                    "{:<12} │{}│ {:>6} ({:>4.1}%) | {} files",
                    extension, bar, Self::format_number(*tokens, &model.config.token_format), percentage, count
                );

                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        // Create title with scroll indicator
        let scroll_title = if ext_vec.len() > content_height {
            format!("{} | Showing {}-{} of {}",
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

        frame.render_widget(extensions_widget, area);
    }

    fn format_number(num: usize, token_format: &code2prompt_core::tokenizer::TokenFormat) -> String {
        use code2prompt_core::tokenizer::TokenFormat;
        use num_format::{SystemLocale, ToFormattedString};
        
        match token_format {
            TokenFormat::Raw => {
                if num >= 1_000_000 {
                    let millions = (num + 500_000) / 1_000_000;
                    format!("{}M", millions)
                } else if num >= 1_000 {
                    let thousands = (num + 500) / 1_000;
                    format!("{}K", thousands)
                } else {
                    format!("{}", num)
                }
            }
            TokenFormat::Format => {
                // Use locale-aware formatting with thousands separators
                if let Ok(locale) = SystemLocale::default() {
                    num.to_formatted_string(&locale)
                } else {
                    // Fallback to raw formatting if locale fails
                    num.to_string()
                }
            }
        }
    }

    fn render_prompt_output_tab_static(model: &Model, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Info bar
                Constraint::Min(0),    // Prompt content
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Simplified status bar - focus only on prompt availability
        let info_text = if model.analysis_in_progress {
            "Generating prompt...".to_string()
        } else if let Some(error) = &model.analysis_error {
            format!("Generation failed: {}", error)
        } else if model.generated_prompt.is_some() {
            "✓ Prompt ready! Copy (C) or Save (S)".to_string()
        } else {
            "Press Enter to generate prompt from selected files".to_string()
        };

        let info_widget = Paragraph::new(info_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Generated Prompt"),
            )
            .style(if model.analysis_error.is_some() {
                Style::default().fg(Color::Red)
            } else if model.analysis_in_progress {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            });
        frame.render_widget(info_widget, layout[0]);

        // Prompt content
        let content = if let Some(prompt) = &model.generated_prompt {
            prompt.clone()
        } else if model.analysis_in_progress {
            "Generating prompt...".to_string()
        } else {
            "Press <Enter> to run analysis and generate prompt.\n\nSelected files will be processed according to your settings.".to_string()
        };

        // Calculate scroll position for display
        let scroll_info = if let Some(prompt) = &model.generated_prompt {
            let total_lines = prompt.lines().count();
            let current_line = model.output_scroll as usize + 1;
            format!("Generated Prompt (Line {}/{})", current_line, total_lines)
        } else {
            "Generated Prompt".to_string()
        };

        let prompt_widget = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(scroll_info))
            .wrap(Wrap { trim: true })
            .scroll((model.output_scroll, 0));
        frame.render_widget(prompt_widget, layout[1]);

        // Controls
        let controls_text = if model.generated_prompt.is_some() {
            "↑↓/PgUp/PgDn: Scroll | C: Copy | S: Save | Enter: Re-run"
        } else {
            "Enter: Run Analysis"
        };

        let controls_widget = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(controls_widget, layout[2]);
    }

    fn render_status_bar_static(model: &Model, frame: &mut Frame, area: Rect) {
        let status_text = if !model.status_message.is_empty() {
            model.status_message.clone()
        } else {
            "Tab/Shift+Tab: Switch tabs | 1/2/3/4: Direct tab | Enter: Run Analysis | Esc/Ctrl+Q: Quit".to_string()
        };

        let status_widget = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(status_widget, area);
    }
}

pub async fn run_tui() -> Result<()> {
    // Parse CLI arguments properly to extract include/exclude patterns
    let args = crate::args::Cli::parse();

    let mut app = TuiApp::new_with_args(
        args.path.clone(),
        args.include.clone(),
        args.exclude.clone(),
    )?;

    let result = app.run().await;

    // Clean up terminal
    restore_terminal()?;

    result
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
