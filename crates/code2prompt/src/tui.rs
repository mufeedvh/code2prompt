//! Terminal User Interface implementation.
//!
//! This module implements the complete TUI for code2prompt using ratatui and crossterm.
//! It provides a tabbed interface with file selection, settings configuration,
//! statistics viewing, and prompt output. The interface supports keyboard navigation,
//! file tree browsing, real-time analysis, and clipboard integration.

use anyhow::Result;
use code2prompt_core::session::Code2PromptSession;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io::{stdout, Stdout};
use tokio::sync::mpsc;

use crate::model::{Message, Model, SettingAction, Tab};
use crate::widgets::*;

use crate::token_map::generate_token_map_with_limit;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}
use crate::utils::build_file_tree_from_session;

pub struct TuiApp {
    model: Model,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    message_tx: mpsc::UnboundedSender<Message>,
    message_rx: mpsc::UnboundedReceiver<Message>,
    input_mode: InputMode,
}

impl TuiApp {
    /// Create a new TUI application with specified parameters.
    ///
    /// Initializes the TUI with a default configuration using the provided path
    /// and file patterns, builds the initial file tree, and sets up the application state.
    ///
    /// # Arguments
    ///
    /// * `path` - Root path of the codebase to analyze
    /// * `include_patterns` - Patterns for files to include
    /// * `exclude_patterns` - Patterns for files to exclude
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The initialized TUI application
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized or the file tree cannot be built.
    pub fn new_with_args(session: Code2PromptSession) -> Result<Self> {
        let terminal = init_terminal()?;
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let model = Model::new_with_cli_args(session);

        Ok(Self {
            model,
            terminal,
            message_tx,
            message_rx,
            input_mode: InputMode::Normal,
        })
    }

    // ~~~ Main Loop ~~~
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

    /// Render the TUI using the provided model and frame.
    ///
    /// This function handles the layout and rendering of all components based on the current state.
    /// It divides the terminal into sections for the tab bar, content area, and status bar,
    /// and renders the appropriate widgets for the active tab.
    ///
    /// # Arguments
    ///
    /// * `model` - The current application state model
    /// * `frame` - The frame to render the UI components onto
    ///
    fn render_with_model(model: &Model, frame: &mut Frame) {
        let area = frame.area();

        // ~~~ Main layout ~~~
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab bar
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status bar
            ])
            .split(area);

        // Tab bar
        Self::render_tab_bar_static(model, frame, main_layout[0]);

        // Current tab content
        match model.current_tab {
            Tab::FileTree => {
                let widget = FileSelectionWidget::new(model);
                let mut state = FileSelectionState::from_model(model);
                frame.render_stateful_widget(widget, main_layout[1], &mut state);
            }
            Tab::Settings => {
                let widget = SettingsWidget::new(model);
                let mut state = SettingsState::from_model(model);
                frame.render_stateful_widget(widget, main_layout[1], &mut state);
            }
            Tab::Statistics => match model.statistics.view {
                crate::model::StatisticsView::Overview => {
                    let widget = StatisticsOverviewWidget::new(model);
                    frame.render_widget(widget, main_layout[1]);
                }
                crate::model::StatisticsView::TokenMap => {
                    let widget = StatisticsTokenMapWidget::new(model);
                    let mut state = TokenMapState::from_model(model);
                    frame.render_stateful_widget(widget, main_layout[1], &mut state);
                }
                crate::model::StatisticsView::Extensions => {
                    let widget = StatisticsByExtensionWidget::new(model);
                    let mut state = ExtensionState::from_model(model);
                    frame.render_stateful_widget(widget, main_layout[1], &mut state);
                }
            },
            Tab::Template => {
                let widget = TemplateWidget::new(model);
                let mut state = TemplateState::from_model(model);
                frame.render_stateful_widget(widget, main_layout[1], &mut state);

                // Synchronize template content back to model if it changed
                // This is a workaround since StatefulWidget doesn't provide a way to get state back
                // In a real implementation, we'd use a different pattern
            }
            Tab::PromptOutput => {
                let widget = OutputWidget::new(model);
                let mut state = OutputState::from_model(model);
                frame.render_stateful_widget(widget, main_layout[1], &mut state);
            }
        }

        // Status bar
        Self::render_status_bar_static(model, frame, main_layout[2]);
    }

    /// Handle a key event and return an optional message.
    ///
    /// This function processes keyboard input, prioritizing search mode
    /// when active. It handles global shortcuts for tab switching and quitting,
    /// as well as delegating tab-specific key events to the appropriate handlers.
    /// # Arguments
    ///
    /// * `key` - The key event to handle.
    ///
    /// # Returns
    ///
    /// * `Option<Message>` - An optional message to be processed by the main loop.
    ///   
    fn handle_key_event(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Check if we're in search mode first - this takes priority over global shortcuts
        if self.input_mode == InputMode::Search && self.model.current_tab == Tab::FileTree {
            return self.handle_file_tree_keys(key);
        }

        // Global shortcuts (only when not in search mode)
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(Message::Quit);
            }
            KeyCode::Esc => return Some(Message::Quit),
            KeyCode::Char('1') => return Some(Message::SwitchTab(Tab::FileTree)),
            KeyCode::Char('2') => return Some(Message::SwitchTab(Tab::Settings)),
            KeyCode::Char('3') => return Some(Message::SwitchTab(Tab::Statistics)),
            KeyCode::Char('4') => return Some(Message::SwitchTab(Tab::Template)),
            KeyCode::Char('5') => return Some(Message::SwitchTab(Tab::PromptOutput)),
            KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Cycle through tabs: Selection -> Settings -> Statistics -> Template -> Output -> Selection
                let next_tab = match self.model.current_tab {
                    Tab::FileTree => Tab::Settings,
                    Tab::Settings => Tab::Statistics,
                    Tab::Statistics => Tab::Template,
                    Tab::Template => Tab::PromptOutput,
                    Tab::PromptOutput => Tab::FileTree,
                };
                return Some(Message::SwitchTab(next_tab));
            }
            KeyCode::BackTab | KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Cycle through tabs in reverse: Selection <- Settings <- Statistics <- Template <- Output <- Selection
                let prev_tab = match self.model.current_tab {
                    Tab::FileTree => Tab::PromptOutput,
                    Tab::Settings => Tab::FileTree,
                    Tab::Statistics => Tab::Settings,
                    Tab::Template => Tab::Statistics,
                    Tab::PromptOutput => Tab::Template,
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
            Tab::Template => self.handle_template_keys(key),
            Tab::PromptOutput => self.handle_prompt_output_keys(key),
        }
    }

    fn handle_file_tree_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Delegate to FileSelectionWidget
        FileSelectionWidget::handle_key_event(
            key,
            &self.model,
            self.input_mode == InputMode::Search,
        )
    }

    fn handle_settings_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Delegate to SettingsWidget
        SettingsWidget::handle_key_event(key, &self.model)
    }

    fn handle_statistics_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Delegate to the appropriate statistics widget based on current view
        match self.model.statistics.view {
            crate::model::StatisticsView::Overview => {
                StatisticsOverviewWidget::handle_key_event(key)
            }
            crate::model::StatisticsView::TokenMap => {
                StatisticsTokenMapWidget::handle_key_event(key)
            }
            crate::model::StatisticsView::Extensions => {
                StatisticsByExtensionWidget::handle_key_event(key)
            }
        }
    }

    fn handle_template_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Create a temporary template state to handle the key event
        let mut temp_state = TemplateState::from_model(&self.model);
        TemplateWidget::handle_key_event(key, &self.model, &mut temp_state)
    }

    fn handle_prompt_output_keys(&self, key: crossterm::event::KeyEvent) -> Option<Message> {
        // Delegate to OutputWidget
        OutputWidget::handle_key_event(key, &self.model)
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
                // Build file tree using session data
                match build_file_tree_from_session(&mut self.model.session.session) {
                    Ok(tree) => {
                        self.model.file_tree.set_file_tree(tree);

                        // File tree will get selection state from session data
                        // No need to maintain separate HashMap

                        self.model.status_message = "File tree refreshed".to_string();
                    }
                    Err(e) => {
                        self.model.status_message = format!("Error loading files: {}", e);
                    }
                }
            }
            Message::UpdateSearchQuery(query) => {
                self.model.file_tree.search_query = query;
                // Reset cursor when search changes
                self.model.file_tree.tree_cursor = 0;
            }
            Message::EnterSearchMode => {
                self.input_mode = InputMode::Search;
                self.model.status_message = "Search mode - Type to search, Esc to exit".to_string();
            }
            Message::ExitSearchMode => {
                self.input_mode = InputMode::Normal;
                self.model.status_message = "Exited search mode".to_string();
            }
            Message::MoveTreeCursor(delta) => {
                let visible_count = self.model.file_tree.get_visible_nodes().len();
                if visible_count > 0 {
                    let new_cursor = if delta > 0 {
                        (self.model.file_tree.tree_cursor + delta as usize).min(visible_count - 1)
                    } else {
                        self.model
                            .file_tree
                            .tree_cursor
                            .saturating_sub((-delta) as usize)
                    };
                    self.model.file_tree.tree_cursor = new_cursor;

                    // Auto-adjust scroll to keep cursor visible using widget method
                    FileSelectionWidget::adjust_file_tree_scroll_for_cursor(
                        self.model.file_tree.tree_cursor,
                        &mut self.model.file_tree.file_tree_scroll,
                        visible_count,
                    );
                }
            }
            Message::MoveSettingsCursor(delta) => {
                let settings_count = self
                    .model
                    .settings
                    .get_settings_items(&self.model.session.session)
                    .len();
                if settings_count > 0 {
                    let new_cursor = if delta > 0 {
                        (self.model.settings.settings_cursor + delta as usize)
                            .min(settings_count - 1)
                    } else {
                        self.model
                            .settings
                            .settings_cursor
                            .saturating_sub((-delta) as usize)
                    };
                    self.model.settings.settings_cursor = new_cursor;
                }
            }
            Message::ToggleFileSelection(index) => {
                let visible_nodes = self.model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    let path = node.path.to_string_lossy().to_string();
                    let name = node.name.clone();
                    let is_directory = node.is_directory;
                    let current = node.is_selected;

                    // Use session methods for file selection instead of direct config manipulation
                    if !current {
                        // Selecting file: use session include_file method
                        self.model.session.session.include_file(node.path.clone());
                    } else {
                        // Deselecting file: use session exclude_file method
                        self.model.session.session.exclude_file(node.path.clone());
                    }

                    // Session methods handle config updates automatically

                    // Update the node in the tree using widget methods
                    if is_directory {
                        FileSelectionWidget::toggle_directory_selection(
                            self.model.file_tree.get_file_tree_mut(),
                            &path,
                            !current,
                        );
                    } else {
                        FileSelectionWidget::update_node_selection(
                            self.model.file_tree.get_file_tree_mut(),
                            &path,
                            !current,
                        );
                    }

                    let action = if current { "Deselected" } else { "Selected" };
                    let extra = if is_directory { " (and contents)" } else { "" };
                    self.model.status_message = format!("{} {}{}", action, name, extra);
                }
            }
            Message::ExpandDirectory(index) => {
                let visible_nodes = self.model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    if node.is_directory {
                        let path = node.path.to_string_lossy().to_string();
                        let name = node.name.clone();
                        FileSelectionWidget::expand_directory(
                            self.model.file_tree.get_file_tree_mut(),
                            &path,
                        );
                        self.model.status_message = format!("Expanded {}", name);
                    }
                }
            }
            Message::CollapseDirectory(index) => {
                let visible_nodes = self.model.file_tree.get_visible_nodes();
                if let Some(node) = visible_nodes.get(index) {
                    if node.is_directory {
                        let path = node.path.to_string_lossy().to_string();
                        let name = node.name.clone();
                        FileSelectionWidget::collapse_directory(
                            self.model.file_tree.get_file_tree_mut(),
                            &path,
                        );
                        self.model.status_message = format!("Collapsed {}", name);
                    }
                }
            }
            Message::ToggleSetting(index) => {
                self.model.settings.update_setting(
                    &mut self.model.session.session,
                    index,
                    SettingAction::Toggle,
                );
                let settings = self
                    .model
                    .settings
                    .get_settings_items(&self.model.session.session);
                if let Some(setting) = settings.get(index) {
                    self.model.status_message = format!("Toggled {}", setting.name);
                }
            }
            Message::CycleSetting(index) => {
                self.model.settings.update_setting(
                    &mut self.model.session.session,
                    index,
                    SettingAction::Cycle,
                );
                let settings = self
                    .model
                    .settings
                    .get_settings_items(&self.model.session.session);
                if let Some(setting) = settings.get(index) {
                    self.model.status_message = format!("Cycled {}", setting.name);
                }
            }
            Message::RunAnalysis => {
                if !self.model.prompt_output.analysis_in_progress {
                    self.model.prompt_output.analysis_in_progress = true;
                    self.model.prompt_output.analysis_error = None;
                    self.model.status_message = "Running analysis...".to_string();

                    // Switch to prompt output tab
                    self.model.current_tab = Tab::PromptOutput;

                    // Run analysis in background using session directly (same as CLI)
                    let mut session = self.model.session.session.clone();
                    let template_content = self.model.template.template_content.clone();
                    let tx = self.message_tx.clone();

                    tokio::spawn(async move {
                        // Set custom template content
                        session.config.template_str = template_content;
                        session.config.template_name = "Custom Template".to_string();

                        match session.generate_prompt() {
                            Ok(rendered) => {
                                // Convert to AnalysisResults format expected by TUI
                                let token_map_entries = if rendered.token_count > 0 {
                                    if let Some(files_value) = session.data.files.as_ref() {
                                        if let Some(files_array) = files_value.as_array() {
                                            generate_token_map_with_limit(
                                                files_array,
                                                rendered.token_count,
                                                Some(50),
                                                Some(0.5),
                                            )
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                };

                                let result = crate::model::AnalysisResults {
                                    file_count: rendered.files.len(),
                                    token_count: Some(rendered.token_count),
                                    generated_prompt: rendered.prompt,
                                    token_map_entries,
                                };
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
                self.model.prompt_output.analysis_in_progress = false;
                self.model.prompt_output.generated_prompt = Some(results.generated_prompt);
                self.model.prompt_output.token_count = results.token_count;
                self.model.prompt_output.file_count = results.file_count;
                self.model.statistics.token_map_entries = results.token_map_entries;
                let tokens = results.token_count.unwrap_or(0);
                self.model.status_message = format!(
                    "Analysis complete! {} tokens, {} files",
                    tokens, results.file_count
                );
            }
            Message::AnalysisError(error) => {
                self.model.prompt_output.analysis_in_progress = false;
                self.model.prompt_output.analysis_error = Some(error.clone());
                self.model.status_message = format!("Analysis failed: {}", error);
            }
            Message::CopyToClipboard => {
                if let Some(prompt) = &self.model.prompt_output.generated_prompt {
                    match crate::clipboard::copy_to_clipboard(prompt) {
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
                if let Some(prompt) = &self.model.prompt_output.generated_prompt {
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
                // Delegate to OutputWidget
                OutputWidget::handle_scroll(
                    delta as i32,
                    &mut self.model.prompt_output.output_scroll,
                    &self.model.prompt_output.generated_prompt,
                );
            }
            Message::ScrollFileTree(delta) => {
                let visible_count = self.model.file_tree.get_visible_nodes().len() as u16;
                let viewport_height = 20; // Approximate viewport height for file tree
                let max_scroll = visible_count.saturating_sub(viewport_height);

                let new_scroll = if delta < 0 {
                    self.model
                        .file_tree
                        .file_tree_scroll
                        .saturating_sub((-delta) as u16)
                } else {
                    self.model
                        .file_tree
                        .file_tree_scroll
                        .saturating_add(delta as u16)
                };

                // Clamp scroll to valid range
                self.model.file_tree.file_tree_scroll = new_scroll.min(max_scroll);
            }
            Message::CycleStatisticsView(direction) => {
                self.model.statistics.view = if direction > 0 {
                    self.model.statistics.view.next()
                } else {
                    self.model.statistics.view.prev()
                };

                self.model.statistics.scroll = 0; // Reset scroll
                self.model.status_message =
                    format!("Switched to {} view", self.model.statistics.view.as_str());
            }
            Message::ScrollStatistics(delta) => {
                // For now, simple scroll logic - will be refined per view
                let new_scroll = if delta < 0 {
                    self.model.statistics.scroll.saturating_sub((-delta) as u16)
                } else {
                    self.model.statistics.scroll.saturating_add(delta as u16)
                };
                self.model.statistics.scroll = new_scroll;
            }
            // Template messages - Redux/Elm style
            Message::ToggleTemplateEdit => {
                self.model.template.template_is_editing = !self.model.template.template_is_editing;
                let status = if self.model.template.template_is_editing {
                    "Edit mode enabled"
                } else {
                    "Edit mode disabled"
                };
                self.model.status_message = status.to_string();
            }
            Message::ScrollTemplate(delta) => {
                let lines_count = self.model.template.template_content.lines().count() as u16;
                let viewport_height = 20; // Approximate viewport height
                let max_scroll = lines_count.saturating_sub(viewport_height);

                let new_scroll = if delta < 0 {
                    self.model
                        .template
                        .template_scroll_offset
                        .saturating_sub((-delta) as u16)
                } else {
                    self.model
                        .template
                        .template_scroll_offset
                        .saturating_add(delta as u16)
                };

                self.model.template.template_scroll_offset = new_scroll.min(max_scroll);
            }
            Message::SaveTemplate(filename) => match self.save_template_with_name(&filename) {
                Ok(_) => {
                    self.model.status_message = format!("Template saved as {}.hbs", filename);
                }
                Err(e) => {
                    self.model.status_message = format!("Save failed: {}", e);
                }
            },
            Message::ReloadTemplate => {
                let template_content = match self.model.session.session.config.output_format {
                    code2prompt_core::template::OutputFormat::Xml => {
                        include_str!("../../code2prompt-core/src/default_template_xml.hbs")
                            .to_string()
                    }
                    _ => include_str!("../../code2prompt-core/src/default_template_md.hbs")
                        .to_string(),
                };
                self.model.template.template_content = template_content;
                self.model.template.template_name = "Default Template".to_string();
                self.model.template.template_cursor_position = 0;
                self.model.template.template_scroll_offset = 0;
                self.model.status_message = "Reloaded default template".to_string();
            }
        }

        Ok(())
    }

    fn render_tab_bar_static(model: &Model, frame: &mut Frame, area: Rect) {
        let tabs = vec![
            "1. Selection",
            "2. Settings",
            "3. Statistics",
            "4. Template",
            "5. Output",
        ];
        let selected = match model.current_tab {
            Tab::FileTree => 0,
            Tab::Settings => 1,
            Tab::Statistics => 2,
            Tab::Template => 3,
            Tab::PromptOutput => 4,
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

    // Template utility methods

    fn save_template_with_name(&self, filename: &str) -> Result<(), String> {
        use std::fs;
        use std::path::PathBuf;

        let templates_dir = PathBuf::from("crates/code2prompt-core/templates");
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)
                .map_err(|e| format!("Failed to create templates directory: {}", e))?;
        }

        let file_path = templates_dir.join(format!("{}.hbs", filename));
        fs::write(&file_path, &self.model.template.template_content)
            .map_err(|e| format!("Failed to save template: {}", e))
    }
}

/// Run the Terminal User Interface.
///
/// This is the main entry point for the TUI mode. It parses command-line arguments,
/// initializes the TUI application, and runs the main event loop until the user exits.
///
/// # Returns
///
/// * `Result<()>` - Ok on successful exit, Err if initialization or runtime errors occur
///
/// # Errors
///
/// Returns an error if the TUI cannot be initialized or if runtime errors occur during execution.
pub async fn run_tui_with_args(session: Code2PromptSession) -> Result<()> {
    let mut app = TuiApp::new_with_args(session)?;

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
