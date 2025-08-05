use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io::{stdout, Stdout};
use tokio::sync::mpsc;
use clap::Parser;

use crate::model::{Message, Model, Tab, SettingAction};
use crate::utils::{run_analysis, build_file_tree};

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
            Tab::PromptOutput => Self::render_prompt_output_tab_static(model, frame, main_layout[1]),
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
            KeyCode::Char('3') => return Some(Message::SwitchTab(Tab::PromptOutput)),
            KeyCode::Tab => {
                // Cycle through tabs: Selection -> Settings -> Output -> Selection
                let next_tab = match self.model.current_tab {
                    Tab::FileTree => Tab::Settings,
                    Tab::Settings => Tab::PromptOutput,
                    Tab::PromptOutput => Tab::FileTree,
                };
                return Some(Message::SwitchTab(next_tab));
            }
            _ => {}
        }
        
        // Tab-specific shortcuts
        match self.model.current_tab {
            Tab::FileTree => self.handle_file_tree_keys(key),
            Tab::Settings => self.handle_settings_keys(key),
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
            KeyCode::Char('r') | KeyCode::Char('R') => Some(Message::RunAnalysis),
            KeyCode::Esc => {
                if !self.model.search_query.is_empty() {
                    Some(Message::UpdateSearchQuery(String::new()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    fn handle_settings_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Up => Some(Message::MoveSettingsCursor(-1)),
            KeyCode::Down => Some(Message::MoveSettingsCursor(1)),
            KeyCode::Char(' ') => Some(Message::ToggleSetting(self.model.settings_cursor)),
            KeyCode::Left | KeyCode::Right => Some(Message::CycleSetting(self.model.settings_cursor)),
            KeyCode::Enter => Some(Message::RunAnalysis),
            _ => None,
        }
    }
    
    fn handle_prompt_output_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Enter => Some(Message::RunAnalysis),
            // Check for Ctrl+Up/Down first for faster scrolling
            KeyCode::Up if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                Some(Message::ScrollOutput(-10))
            }
            KeyCode::Down if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
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
                        Self::sync_selected_files_from_tree(&self.model.file_tree, &mut self.model.selected_files);
                        
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
                    let current = self.model.selected_files.get(&path).copied().unwrap_or(node.is_selected);
                    
                    // Update the node in the tree (and recursively if it's a directory)
                    if is_directory {
                        self.toggle_directory_selection(&path, !current);
                    } else {
                        self.model.selected_files.insert(path.clone(), !current);
                        self.update_node_selection(&path, !current);
                    }
                    
                    let action = if current { "Deselected" } else { "Selected" };
                    let extra = if is_directory { " (and contents)" } else { "" };
                    self.model.status_message = format!("{} {}{}", action, name, extra);
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
                                let _ = tx.send(Message::AnalysisComplete(
                                    result.generated_prompt,
                                    result.token_count.unwrap_or(0),
                                    result.file_count,
                                ));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::AnalysisError(e.to_string()));
                            }
                        }
                    });
                }
            }
            Message::AnalysisComplete(prompt, tokens, files) => {
                self.model.analysis_in_progress = false;
                self.model.generated_prompt = Some(prompt);
                self.model.token_count = Some(tokens);
                self.model.file_count = files;
                self.model.status_message = format!("Analysis complete! {} tokens, {} files", tokens, files);
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
        }
        
        Ok(())
    }
    
    // Helper methods for tree manipulation
    fn sync_selected_files_from_tree(nodes: &[crate::model::FileNode], selected_files: &mut std::collections::HashMap<String, bool>) {
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
        Self::toggle_directory_children_selection(&mut self.model.file_tree, path, selected, &mut self.model.selected_files);
    }
    
    fn toggle_directory_children_selection(
        nodes: &mut [crate::model::FileNode], 
        dir_path: &str, 
        selected: bool,
        selected_files: &mut std::collections::HashMap<String, bool>
    ) {
        for node in nodes.iter_mut() {
            if node.path.to_string_lossy() == dir_path && node.is_directory {
                // Found the directory, now update all its children recursively
                Self::select_all_children(&mut node.children, selected, selected_files);
                return;
            }
            Self::toggle_directory_children_selection(&mut node.children, dir_path, selected, selected_files);
        }
    }
    
    fn select_all_children(
        nodes: &mut [crate::model::FileNode], 
        selected: bool,
        selected_files: &mut std::collections::HashMap<String, bool>
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
    
    fn update_node_selection_recursive(nodes: &mut [crate::model::FileNode], path: &str, selected: bool) -> bool {
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
                    if let Ok(children) = crate::utils::load_directory_children(&node.path, node.level + 1) {
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
    
    fn render_tab_bar_static(model: &Model, frame: &mut Frame, area: Rect) {
        let tabs = vec!["1. Selection", "2. Settings", "3. Output"];
        let selected = match model.current_tab {
            Tab::FileTree => 0,
            Tab::Settings => 1,
            Tab::PromptOutput => 2,
        };
        
        let tabs_widget = Tabs::new(tabs)
            .block(Block::default().borders(Borders::ALL).title("Code2Prompt TUI"))
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        
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
        
        // File tree
        let visible_nodes = model.get_visible_nodes();
        let items: Vec<ListItem> = visible_nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let indent = "  ".repeat(node.level);
                let icon = if node.is_directory {
                    if node.is_expanded { "📂" } else { "📁" }
                } else {
                    "📄"
                };
                let checkbox = if node.is_selected { "☑" } else { "☐" };
                
                let content = format!("{}{} {} {}", indent, icon, checkbox, node.name);
                let mut style = Style::default();
                
                if i == model.tree_cursor {
                    style = style.bg(Color::Blue).fg(Color::White);
                }
                
                if node.is_selected {
                    style = style.fg(Color::Green);
                }
                
                ListItem::new(content).style(style)
            })
            .collect();
        
        let tree_widget = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Files ({})", visible_nodes.len())))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
        
        frame.render_widget(tree_widget, layout[0]);
        
        // Search bar (moved below tree)
        let search_widget = Paragraph::new(model.search_query.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Search (type to filter)"))
            .style(Style::default().fg(Color::Green));
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
            .block(Block::default().borders(Borders::ALL).title("Filter Patterns"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(pattern_widget, layout[2]);
        
        // Instructions
        let instructions = Paragraph::new(
            "↑↓: Navigate | Space: Select/Deselect | ←→: Expand/Collapse | Enter: Run Analysis | Type to Search"
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
            items.push(ListItem::new(format!("── {} ──", group.name))
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            
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
                
                let content = format!("  {}: {} - {}", item.name, value_display, item.description);
                let mut style = Style::default();
                
                if item_index == model.settings_cursor {
                    style = style.bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
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
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Settings"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
        
        frame.render_widget(settings_widget, layout[0]);
        
        // Instructions
        let instructions = Paragraph::new(
            "↑↓: Navigate | Space: Toggle | ←→: Cycle Options | Enter: Run Analysis"
        )
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, layout[1]);
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
        
        // Info bar
        let info_text = if let Some(token_count) = model.token_count {
            let selected_count = model.selected_files.values().filter(|&&v| v).count();
            format!("Files: {} selected ({} total) | Tokens: {} | Status: Ready", 
                   selected_count, model.file_count, token_count)
        } else if model.analysis_in_progress {
            "Analysis in progress...".to_string()
        } else if let Some(error) = &model.analysis_error {
            format!("Error: {}", error)
        } else {
            let selected_count = model.selected_files.values().filter(|&&v| v).count();
            if selected_count > 0 {
                format!("Ready: {} files selected. Press Enter to run analysis.", selected_count)
            } else {
                "No files selected. Select files in the Selection tab first.".to_string()
            }
        };
        
        let info_widget = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Analysis Info"))
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
            "Press R to run analysis and generate prompt.\n\nSelected files will be processed according to your settings.".to_string()
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
            "Tab: Next tab | 1/2/3: Direct tab | Enter: Run Analysis | Esc/Ctrl+Q: Quit".to_string()
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
        args.exclude.clone()
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