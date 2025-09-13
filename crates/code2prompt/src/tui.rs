//! Terminal User Interface implementation.
//!
//! This module implements the complete TUI for code2prompt using ratatui and crossterm.
//! It provides a tabbed interface with file selection, settings configuration,
//! statistics viewing, and prompt output. The interface supports keyboard navigation,
//! file tree browsing, real-time analysis, and clipboard integration.

use anyhow::Result;
use code2prompt_core::session::Code2PromptSession;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    widgets::*,
};
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
    pub fn new(session: Code2PromptSession) -> Result<Self> {
        let terminal = init_terminal()?;
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let model = Model::new(session);

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
                                // Convert crossterm KeyEvent to ratatui KeyEvent
                                let ratatui_key = KeyEvent {
                                    code: match key.code {
                                        crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
                                        crossterm::event::KeyCode::Enter => KeyCode::Enter,
                                        crossterm::event::KeyCode::Left => KeyCode::Left,
                                        crossterm::event::KeyCode::Right => KeyCode::Right,
                                        crossterm::event::KeyCode::Up => KeyCode::Up,
                                        crossterm::event::KeyCode::Down => KeyCode::Down,
                                        crossterm::event::KeyCode::Home => KeyCode::Home,
                                        crossterm::event::KeyCode::End => KeyCode::End,
                                        crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
                                        crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
                                        crossterm::event::KeyCode::Tab => KeyCode::Tab,
                                        crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
                                        crossterm::event::KeyCode::Delete => KeyCode::Delete,
                                        crossterm::event::KeyCode::Insert => KeyCode::Insert,
                                        crossterm::event::KeyCode::F(n) => KeyCode::F(n),
                                        crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
                                        crossterm::event::KeyCode::Null => KeyCode::Null,
                                        crossterm::event::KeyCode::Esc => KeyCode::Esc,
                                        crossterm::event::KeyCode::CapsLock => KeyCode::CapsLock,
                                        crossterm::event::KeyCode::ScrollLock => KeyCode::ScrollLock,
                                        crossterm::event::KeyCode::NumLock => KeyCode::NumLock,
                                        crossterm::event::KeyCode::PrintScreen => KeyCode::PrintScreen,
                                        crossterm::event::KeyCode::Pause => KeyCode::Pause,
                                        crossterm::event::KeyCode::Menu => KeyCode::Menu,
                                        crossterm::event::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
                                        crossterm::event::KeyCode::Media(media) => KeyCode::Media(match media {
                                            crossterm::event::MediaKeyCode::Play => ratatui::crossterm::event::MediaKeyCode::Play,
                                            crossterm::event::MediaKeyCode::Pause => ratatui::crossterm::event::MediaKeyCode::Pause,
                                            crossterm::event::MediaKeyCode::PlayPause => ratatui::crossterm::event::MediaKeyCode::PlayPause,
                                            crossterm::event::MediaKeyCode::Reverse => ratatui::crossterm::event::MediaKeyCode::Reverse,
                                            crossterm::event::MediaKeyCode::Stop => ratatui::crossterm::event::MediaKeyCode::Stop,
                                            crossterm::event::MediaKeyCode::FastForward => ratatui::crossterm::event::MediaKeyCode::FastForward,
                                            crossterm::event::MediaKeyCode::Rewind => ratatui::crossterm::event::MediaKeyCode::Rewind,
                                            crossterm::event::MediaKeyCode::TrackNext => ratatui::crossterm::event::MediaKeyCode::TrackNext,
                                            crossterm::event::MediaKeyCode::TrackPrevious => ratatui::crossterm::event::MediaKeyCode::TrackPrevious,
                                            crossterm::event::MediaKeyCode::Record => ratatui::crossterm::event::MediaKeyCode::Record,
                                            crossterm::event::MediaKeyCode::LowerVolume => ratatui::crossterm::event::MediaKeyCode::LowerVolume,
                                            crossterm::event::MediaKeyCode::RaiseVolume => ratatui::crossterm::event::MediaKeyCode::RaiseVolume,
                                            crossterm::event::MediaKeyCode::MuteVolume => ratatui::crossterm::event::MediaKeyCode::MuteVolume,
                                        }),
                                        crossterm::event::KeyCode::Modifier(modifier) => KeyCode::Modifier(match modifier {
                                            crossterm::event::ModifierKeyCode::LeftShift => ratatui::crossterm::event::ModifierKeyCode::LeftShift,
                                            crossterm::event::ModifierKeyCode::LeftControl => ratatui::crossterm::event::ModifierKeyCode::LeftControl,
                                            crossterm::event::ModifierKeyCode::LeftAlt => ratatui::crossterm::event::ModifierKeyCode::LeftAlt,
                                            crossterm::event::ModifierKeyCode::LeftSuper => ratatui::crossterm::event::ModifierKeyCode::LeftSuper,
                                            crossterm::event::ModifierKeyCode::LeftHyper => ratatui::crossterm::event::ModifierKeyCode::LeftHyper,
                                            crossterm::event::ModifierKeyCode::LeftMeta => ratatui::crossterm::event::ModifierKeyCode::LeftMeta,
                                            crossterm::event::ModifierKeyCode::RightShift => ratatui::crossterm::event::ModifierKeyCode::RightShift,
                                            crossterm::event::ModifierKeyCode::RightControl => ratatui::crossterm::event::ModifierKeyCode::RightControl,
                                            crossterm::event::ModifierKeyCode::RightAlt => ratatui::crossterm::event::ModifierKeyCode::RightAlt,
                                            crossterm::event::ModifierKeyCode::RightSuper => ratatui::crossterm::event::ModifierKeyCode::RightSuper,
                                            crossterm::event::ModifierKeyCode::RightHyper => ratatui::crossterm::event::ModifierKeyCode::RightHyper,
                                            crossterm::event::ModifierKeyCode::RightMeta => ratatui::crossterm::event::ModifierKeyCode::RightMeta,
                                            crossterm::event::ModifierKeyCode::IsoLevel3Shift => ratatui::crossterm::event::ModifierKeyCode::IsoLevel3Shift,
                                            crossterm::event::ModifierKeyCode::IsoLevel5Shift => ratatui::crossterm::event::ModifierKeyCode::IsoLevel5Shift,
                                        }),
                                    },
                                    modifiers: KeyModifiers::from_bits_truncate(key.modifiers.bits()),
                                    kind: match key.kind {
                                        crossterm::event::KeyEventKind::Press => ratatui::crossterm::event::KeyEventKind::Press,
                                        crossterm::event::KeyEventKind::Repeat => ratatui::crossterm::event::KeyEventKind::Repeat,
                                        crossterm::event::KeyEventKind::Release => ratatui::crossterm::event::KeyEventKind::Release,
                                    },
                                    state: ratatui::crossterm::event::KeyEventState::from_bits_truncate(key.state.bits()),
                                };

                                if let Some(message) = self.handle_key_event(ratatui_key) {
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
    fn handle_key_event(&self, key: KeyEvent) -> Option<Message> {
        // Check if we're in search mode first - this takes priority over global shortcuts
        if self.input_mode == InputMode::Search && self.model.current_tab == Tab::FileTree {
            return self.handle_file_tree_keys(key);
        }

        // Check if we're in template editing mode - ESC should exit editing mode, not quit app
        if self.model.current_tab == Tab::Template && self.model.template.is_in_editing_mode() {
            if key.code == KeyCode::Esc {
                return Some(Message::SetTemplateFocusMode(
                    crate::model::template::FocusMode::Normal,
                ));
            }
            // In editing modes, delegate to template handler
            return self.handle_template_keys(key);
        }

        // Global shortcuts (only when not in search mode or template editing mode)
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

    /// Convert ratatui KeyEvent to crossterm KeyEvent for compatibility with other widgets
    fn convert_to_crossterm_key(&self, key: KeyEvent) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent {
            code: match key.code {
                KeyCode::Backspace => crossterm::event::KeyCode::Backspace,
                KeyCode::Enter => crossterm::event::KeyCode::Enter,
                KeyCode::Left => crossterm::event::KeyCode::Left,
                KeyCode::Right => crossterm::event::KeyCode::Right,
                KeyCode::Up => crossterm::event::KeyCode::Up,
                KeyCode::Down => crossterm::event::KeyCode::Down,
                KeyCode::Home => crossterm::event::KeyCode::Home,
                KeyCode::End => crossterm::event::KeyCode::End,
                KeyCode::PageUp => crossterm::event::KeyCode::PageUp,
                KeyCode::PageDown => crossterm::event::KeyCode::PageDown,
                KeyCode::Tab => crossterm::event::KeyCode::Tab,
                KeyCode::BackTab => crossterm::event::KeyCode::BackTab,
                KeyCode::Delete => crossterm::event::KeyCode::Delete,
                KeyCode::Insert => crossterm::event::KeyCode::Insert,
                KeyCode::F(n) => crossterm::event::KeyCode::F(n),
                KeyCode::Char(c) => crossterm::event::KeyCode::Char(c),
                KeyCode::Null => crossterm::event::KeyCode::Null,
                KeyCode::Esc => crossterm::event::KeyCode::Esc,
                KeyCode::CapsLock => crossterm::event::KeyCode::CapsLock,
                KeyCode::ScrollLock => crossterm::event::KeyCode::ScrollLock,
                KeyCode::NumLock => crossterm::event::KeyCode::NumLock,
                KeyCode::PrintScreen => crossterm::event::KeyCode::PrintScreen,
                KeyCode::Pause => crossterm::event::KeyCode::Pause,
                KeyCode::Menu => crossterm::event::KeyCode::Menu,
                KeyCode::KeypadBegin => crossterm::event::KeyCode::KeypadBegin,
                KeyCode::Media(media) => crossterm::event::KeyCode::Media(match media {
                    ratatui::crossterm::event::MediaKeyCode::Play => {
                        crossterm::event::MediaKeyCode::Play
                    }
                    ratatui::crossterm::event::MediaKeyCode::Pause => {
                        crossterm::event::MediaKeyCode::Pause
                    }
                    ratatui::crossterm::event::MediaKeyCode::PlayPause => {
                        crossterm::event::MediaKeyCode::PlayPause
                    }
                    ratatui::crossterm::event::MediaKeyCode::Reverse => {
                        crossterm::event::MediaKeyCode::Reverse
                    }
                    ratatui::crossterm::event::MediaKeyCode::Stop => {
                        crossterm::event::MediaKeyCode::Stop
                    }
                    ratatui::crossterm::event::MediaKeyCode::FastForward => {
                        crossterm::event::MediaKeyCode::FastForward
                    }
                    ratatui::crossterm::event::MediaKeyCode::Rewind => {
                        crossterm::event::MediaKeyCode::Rewind
                    }
                    ratatui::crossterm::event::MediaKeyCode::TrackNext => {
                        crossterm::event::MediaKeyCode::TrackNext
                    }
                    ratatui::crossterm::event::MediaKeyCode::TrackPrevious => {
                        crossterm::event::MediaKeyCode::TrackPrevious
                    }
                    ratatui::crossterm::event::MediaKeyCode::Record => {
                        crossterm::event::MediaKeyCode::Record
                    }
                    ratatui::crossterm::event::MediaKeyCode::LowerVolume => {
                        crossterm::event::MediaKeyCode::LowerVolume
                    }
                    ratatui::crossterm::event::MediaKeyCode::RaiseVolume => {
                        crossterm::event::MediaKeyCode::RaiseVolume
                    }
                    ratatui::crossterm::event::MediaKeyCode::MuteVolume => {
                        crossterm::event::MediaKeyCode::MuteVolume
                    }
                }),
                KeyCode::Modifier(modifier) => {
                    crossterm::event::KeyCode::Modifier(match modifier {
                        ratatui::crossterm::event::ModifierKeyCode::LeftShift => {
                            crossterm::event::ModifierKeyCode::LeftShift
                        }
                        ratatui::crossterm::event::ModifierKeyCode::LeftControl => {
                            crossterm::event::ModifierKeyCode::LeftControl
                        }
                        ratatui::crossterm::event::ModifierKeyCode::LeftAlt => {
                            crossterm::event::ModifierKeyCode::LeftAlt
                        }
                        ratatui::crossterm::event::ModifierKeyCode::LeftSuper => {
                            crossterm::event::ModifierKeyCode::LeftSuper
                        }
                        ratatui::crossterm::event::ModifierKeyCode::LeftHyper => {
                            crossterm::event::ModifierKeyCode::LeftHyper
                        }
                        ratatui::crossterm::event::ModifierKeyCode::LeftMeta => {
                            crossterm::event::ModifierKeyCode::LeftMeta
                        }
                        ratatui::crossterm::event::ModifierKeyCode::RightShift => {
                            crossterm::event::ModifierKeyCode::RightShift
                        }
                        ratatui::crossterm::event::ModifierKeyCode::RightControl => {
                            crossterm::event::ModifierKeyCode::RightControl
                        }
                        ratatui::crossterm::event::ModifierKeyCode::RightAlt => {
                            crossterm::event::ModifierKeyCode::RightAlt
                        }
                        ratatui::crossterm::event::ModifierKeyCode::RightSuper => {
                            crossterm::event::ModifierKeyCode::RightSuper
                        }
                        ratatui::crossterm::event::ModifierKeyCode::RightHyper => {
                            crossterm::event::ModifierKeyCode::RightHyper
                        }
                        ratatui::crossterm::event::ModifierKeyCode::RightMeta => {
                            crossterm::event::ModifierKeyCode::RightMeta
                        }
                        ratatui::crossterm::event::ModifierKeyCode::IsoLevel3Shift => {
                            crossterm::event::ModifierKeyCode::IsoLevel3Shift
                        }
                        ratatui::crossterm::event::ModifierKeyCode::IsoLevel5Shift => {
                            crossterm::event::ModifierKeyCode::IsoLevel5Shift
                        }
                    })
                }
            },
            modifiers: crossterm::event::KeyModifiers::from_bits_truncate(key.modifiers.bits()),
            kind: match key.kind {
                ratatui::crossterm::event::KeyEventKind::Press => {
                    crossterm::event::KeyEventKind::Press
                }
                ratatui::crossterm::event::KeyEventKind::Repeat => {
                    crossterm::event::KeyEventKind::Repeat
                }
                ratatui::crossterm::event::KeyEventKind::Release => {
                    crossterm::event::KeyEventKind::Release
                }
            },
            state: crossterm::event::KeyEventState::from_bits_truncate(key.state.bits()),
        }
    }

    fn handle_file_tree_keys(&self, key: KeyEvent) -> Option<Message> {
        // Convert to crossterm KeyEvent for compatibility
        let crossterm_key = self.convert_to_crossterm_key(key);
        // Delegate to FileSelectionWidget
        FileSelectionWidget::handle_key_event(
            crossterm_key,
            &self.model,
            self.input_mode == InputMode::Search,
        )
    }

    fn handle_settings_keys(&self, key: KeyEvent) -> Option<Message> {
        // Convert to crossterm KeyEvent for compatibility
        let crossterm_key = self.convert_to_crossterm_key(key);
        // Delegate to SettingsWidget
        SettingsWidget::handle_key_event(crossterm_key, &self.model)
    }

    fn handle_statistics_keys(&self, key: KeyEvent) -> Option<Message> {
        // Delegate to the appropriate statistics widget based on current view
        match self.model.statistics.view {
            crate::model::StatisticsView::Overview => {
                StatisticsOverviewWidget::handle_key_event(key)
            }
            crate::model::StatisticsView::TokenMap => {
                // Convert to crossterm KeyEvent for compatibility with other widgets
                let crossterm_key = self.convert_to_crossterm_key(key);
                StatisticsTokenMapWidget::handle_key_event(crossterm_key)
            }
            crate::model::StatisticsView::Extensions => {
                // Convert to crossterm KeyEvent for compatibility with other widgets
                let crossterm_key = self.convert_to_crossterm_key(key);
                StatisticsByExtensionWidget::handle_key_event(crossterm_key)
            }
        }
    }

    fn handle_template_keys(&self, key: KeyEvent) -> Option<Message> {
        // This is a temporary workaround - in a proper implementation,
        // we'd need to pass the state back to the model
        // For now, we'll handle template keys directly here

        // Check if we're in editing mode by looking at the model's template state
        let is_in_editing_mode = self.model.template.is_in_editing_mode();
        let current_focus = self.model.template.get_focus();

        // Handle ESC key to exit editing modes
        if key.code == KeyCode::Esc && is_in_editing_mode {
            return Some(Message::SetTemplateFocusMode(
                crate::model::template::FocusMode::Normal,
            ));
        }

        // In editing modes, handle keys directly
        if is_in_editing_mode {
            match current_focus {
                crate::model::template::TemplateFocus::Editor => {
                    // For editor, we need to pass the key to the textarea
                    return Some(Message::TemplateEditorInput(key));
                }
                crate::model::template::TemplateFocus::Variables => {
                    // Handle variable navigation and editing
                    return Some(Message::TemplateVariableInput(key));
                }
                _ => {}
            }
        }

        // Normal mode: Handle global shortcuts and focus switching
        match key.code {
            KeyCode::Char('e') | KeyCode::Char('E') => {
                return Some(Message::SetTemplateFocus(
                    crate::model::template::TemplateFocus::Editor,
                    crate::model::template::FocusMode::EditingTemplate,
                ));
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                return Some(Message::SetTemplateFocus(
                    crate::model::template::TemplateFocus::Variables,
                    crate::model::template::FocusMode::EditingVariable,
                ));
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                return Some(Message::SetTemplateFocus(
                    crate::model::template::TemplateFocus::Picker,
                    crate::model::template::FocusMode::Normal,
                ));
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Save template with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("custom_template_{}", timestamp);
                return Some(Message::SaveTemplate(filename));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reload default template
                return Some(Message::ReloadTemplate);
            }
            KeyCode::Enter => {
                // Run analysis
                return Some(Message::RunAnalysis);
            }
            _ => {}
        }

        // Handle input for focused component in normal mode
        if current_focus == crate::model::template::TemplateFocus::Picker {
            // Handle picker navigation - Tab should navigate within picker, not change tabs
            match key.code {
                KeyCode::Up => return Some(Message::TemplatePickerMove(-1)),
                KeyCode::Down => return Some(Message::TemplatePickerMove(1)),
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Char('L') => {
                    return Some(Message::LoadTemplate);
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    return Some(Message::RefreshTemplates);
                }
                _ => {}
            }
        }

        None
    }

    fn handle_prompt_output_keys(&self, key: KeyEvent) -> Option<Message> {
        // Convert to crossterm KeyEvent for compatibility
        let crossterm_key = self.convert_to_crossterm_key(key);
        // Delegate to OutputWidget
        OutputWidget::handle_key_event(crossterm_key, &self.model)
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
                    let template_content = self.model.template.get_template_content().to_string();
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
            // Template messages - delegate to template state
            Message::SaveTemplate(ref _filename) => {
                // Delegate to template state
                if let Some(result_msg) = self.model.template.handle_message(&message) {
                    self.handle_message(result_msg)?;
                }
            }
            Message::ReloadTemplate => {
                // Delegate to template state
                if let Some(result_msg) = self.model.template.handle_message(&message) {
                    self.handle_message(result_msg)?;
                }
            }
            Message::LoadTemplate => {
                // Delegate to template state
                if let Some(result_msg) = self.model.template.handle_message(&message) {
                    self.handle_message(result_msg)?;
                }
            }
            Message::RefreshTemplates => {
                // Delegate to template state
                if let Some(result_msg) = self.model.template.handle_message(&message) {
                    self.handle_message(result_msg)?;
                }
            }
            // Template focus and input handling
            Message::SetTemplateFocus(focus, mode) => {
                self.model.template.set_focus(focus);
                self.model.template.set_focus_mode(mode);
                if mode == crate::model::template::FocusMode::EditingVariable {
                    self.model
                        .template
                        .variables
                        .move_to_first_missing_variable();
                }
                self.model.status_message = format!("Template focus: {:?} ({:?})", focus, mode);
            }
            Message::SetTemplateFocusMode(mode) => {
                self.model.template.set_focus_mode(mode);
                self.model.status_message = format!("Template mode: {:?}", mode);
            }
            Message::TemplateEditorInput(key) => {
                // Pass key directly to the textarea
                self.model.template.editor.editor.input(key);
                self.model.template.editor.sync_content_from_textarea();
                self.model.template.editor.validate_template();
                self.model.template.sync_variables_with_template();
            }
            Message::TemplateVariableInput(key) => {
                // Handle variable input
                let variables = self.model.template.get_organized_variables();
                crate::widgets::template::TemplateVariableWidget::handle_key_event(
                    key,
                    &mut self.model.template.variables,
                    &variables,
                    true,
                );
                self.model.template.sync_variables_with_template();
            }
            Message::TemplatePickerMove(delta) => {
                if delta > 0 {
                    self.model.template.picker.move_cursor_down();
                } else {
                    self.model.template.picker.move_cursor_up();
                }
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
    let mut app = TuiApp::new(session)?;

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
