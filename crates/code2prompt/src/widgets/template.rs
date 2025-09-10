//! Template widget for editing and managing Handlebars templates.
//!
//! This widget provides a text editor interface for modifying templates,
//! loading templates from files, and saving custom templates with variable management.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
/// State for the template widget
#[derive(Debug, Clone)]
pub struct TemplateState {
    pub template_content: String,
    pub is_editing: bool,
    pub available_templates: Vec<TemplateFile>,
    pub template_list_cursor: usize,
    pub show_template_list: bool,
    pub status_message: String,
    pub current_template_name: String,
    pub user_variables: HashMap<String, String>,
    pub current_tab: TemplateTab,
    pub variable_list_cursor: usize,
    pub editing_variable_key: Option<String>,
    pub editing_variable_value: Option<String>,
    pub variable_key_content: String,
    pub variable_value_content: String,
    pub show_variable_editor: bool,
}

/// Template tabs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateTab {
    Editor,
    Variables,
}

/// Represents a template file
#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
}

impl TemplateState {
    pub fn from_model(model: &Model) -> Self {
        let mut state = Self {
            template_content: model.template.template_content.clone(),
            is_editing: false,
            available_templates: Vec::new(),
            template_list_cursor: 0,
            show_template_list: false,
            status_message: String::new(),
            current_template_name: "Default".to_string(),
            user_variables: model.session.session.config.user_variables.clone(),
            current_tab: TemplateTab::Editor,
            variable_list_cursor: 0,
            editing_variable_key: None,
            editing_variable_value: None,
            variable_key_content: String::new(),
            variable_value_content: String::new(),
            show_variable_editor: false,
        };

        // Load available templates from directory
        state.load_available_templates();
        state
    }

    fn load_available_templates(&mut self) {
        self.available_templates.clear();

        // Add default templates
        self.available_templates.push(TemplateFile {
            name: "Default (Markdown)".to_string(),
            path: PathBuf::new(),
            is_default: true,
        });

        self.available_templates.push(TemplateFile {
            name: "Default (XML)".to_string(),
            path: PathBuf::new(),
            is_default: true,
        });

        // Load built-in templates from the templates directory
        let templates_dir = PathBuf::from("crates/code2prompt-core/templates");
        if templates_dir.exists() {
            if let Ok(entries) = fs::read_dir(&templates_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            self.available_templates.push(TemplateFile {
                                name: format!(
                                    "Built-in: {}",
                                    name.replace('-', " ").replace('_', " ")
                                ),
                                path: path.clone(),
                                is_default: false,
                            });
                        }
                    }
                }
            }
        }

        // Load user templates from the user data directory
        if let Ok(user_templates) = crate::utils::load_user_templates() {
            for (name, path) in user_templates {
                self.available_templates.push(TemplateFile {
                    name: format!("User: {}", name.replace('-', " ").replace('_', " ")),
                    path,
                    is_default: false,
                });
            }
        }
    }

    pub fn load_template_from_file(&mut self, template_file: &TemplateFile) -> Result<(), String> {
        let content = if template_file.is_default {
            match template_file.name.as_str() {
                "Default (Markdown)" => {
                    include_str!("../../../code2prompt-core/src/default_template_md.hbs")
                        .to_string()
                }
                "Default (XML)" => {
                    include_str!("../../../code2prompt-core/src/default_template_xml.hbs")
                        .to_string()
                }
                _ => return Err("Unknown default template".to_string()),
            }
        } else {
            match fs::read_to_string(&template_file.path) {
                Ok(content) => content,
                Err(e) => return Err(format!("Failed to load template: {}", e)),
            }
        };

        self.template_content = content.clone();
        self.current_template_name = template_file.name.clone();
        Ok(())
    }

    pub fn get_default_variables(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "absolute_code_path",
                "Absolute path to the codebase directory",
            ),
            ("source_tree", "Directory tree structure of the codebase"),
            ("files", "Array of file objects with content and metadata"),
            ("git_diff", "Git diff output (if enabled)"),
            (
                "git_diff_branch",
                "Git diff between branches (if configured)",
            ),
            ("git_log_branch", "Git log between branches (if configured)"),
        ]
    }

    pub fn add_user_variable(&mut self, key: String, value: String) {
        self.user_variables.insert(key, value);
    }

    pub fn remove_user_variable(&mut self, key: &str) {
        self.user_variables.remove(key);
    }

    pub fn get_user_variables_list(&self) -> Vec<(String, String)> {
        self.user_variables
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

/// Template widget for editing Handlebars templates
pub struct TemplateWidget;

impl TemplateWidget {
    pub fn new(_model: &Model) -> Self {
        Self
    }

    pub fn handle_key_event(
        key: KeyEvent,
        _model: &Model,
        state: &mut TemplateState,
    ) -> Option<Message> {
        if state.show_template_list {
            return Self::handle_template_list_keys(key, state);
        }

        if state.show_variable_editor {
            return Self::handle_variable_editor_keys(key, state);
        }

        // Global shortcuts
        match key.code {
            KeyCode::Tab => {
                state.current_tab = match state.current_tab {
                    TemplateTab::Editor => TemplateTab::Variables,
                    TemplateTab::Variables => TemplateTab::Editor,
                };
                None
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                state.show_template_list = true;
                None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Save template with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("custom_template_{}", timestamp);
                Some(Message::SaveTemplate(filename))
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reload default template
                Some(Message::ReloadTemplate)
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Run analysis with current template
                Some(Message::RunAnalysis)
            }
            _ => match state.current_tab {
                TemplateTab::Editor => Self::handle_editor_keys(key, state),
                TemplateTab::Variables => Self::handle_variables_keys(key, state),
            },
        }
    }

    fn handle_editor_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Char('e') | KeyCode::Char('E') => {
                state.is_editing = !state.is_editing;
                Some(Message::ToggleTemplateEdit)
            }
            _ if state.is_editing => {
                // Handle basic text editing for template content
                match key.code {
                    KeyCode::Char(c) => {
                        state.template_content.push(c);
                    }
                    KeyCode::Backspace => {
                        state.template_content.pop();
                    }
                    KeyCode::Enter => {
                        state.template_content.push('\n');
                    }
                    _ => {}
                }
                None
            }
            KeyCode::Up => Some(Message::ScrollTemplate(-1)),
            KeyCode::Down => Some(Message::ScrollTemplate(1)),
            KeyCode::PageUp => Some(Message::ScrollTemplate(-10)),
            KeyCode::PageDown => Some(Message::ScrollTemplate(10)),
            _ => None,
        }
    }

    fn handle_variables_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Up => {
                if state.variable_list_cursor > 0 {
                    state.variable_list_cursor -= 1;
                }
                None
            }
            KeyCode::Down => {
                let max_cursor = state.get_default_variables().len() + state.user_variables.len();
                if state.variable_list_cursor < max_cursor.saturating_sub(1) {
                    state.variable_list_cursor += 1;
                }
                None
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Add new variable
                state.show_variable_editor = true;
                state.editing_variable_key = Some(String::new());
                state.editing_variable_value = Some(String::new());
                state.variable_key_content = String::new();
                state.variable_value_content = String::new();
                None
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                // Delete selected user variable
                let default_vars_count = state.get_default_variables().len();
                if state.variable_list_cursor >= default_vars_count {
                    let user_var_index = state.variable_list_cursor - default_vars_count;
                    let user_vars: Vec<_> = state.user_variables.keys().cloned().collect();
                    if let Some(key) = user_vars.get(user_var_index) {
                        state.remove_user_variable(key);
                        state.status_message = format!("Deleted variable: {}", key);
                        if state.variable_list_cursor > 0 {
                            state.variable_list_cursor -= 1;
                        }
                    }
                }
                None
            }
            KeyCode::Enter => {
                // Edit selected user variable
                let default_vars_count = state.get_default_variables().len();
                if state.variable_list_cursor >= default_vars_count {
                    let user_var_index = state.variable_list_cursor - default_vars_count;
                    let user_vars: Vec<_> = state.get_user_variables_list();
                    if let Some((key, value)) = user_vars.get(user_var_index) {
                        state.show_variable_editor = true;
                        state.editing_variable_key = Some(key.clone());
                        state.editing_variable_value = Some(value.clone());
                        state.variable_key_content = key.clone();
                        state.variable_value_content = value.clone();
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn handle_variable_editor_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Esc => {
                state.show_variable_editor = false;
                state.editing_variable_key = None;
                state.editing_variable_value = None;
                None
            }
            KeyCode::Tab => {
                // Switch focus between key and value inputs
                // For simplicity, we'll just handle both inputs
                None
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save variable
                let key = state.variable_key_content.clone();
                let value = state.variable_value_content.clone();

                if !key.is_empty() {
                    state.add_user_variable(key.clone(), value.clone());
                    state.status_message = format!("Saved variable: {} = {}", key, value);
                    state.show_variable_editor = false;
                    state.editing_variable_key = None;
                    state.editing_variable_value = None;
                }
                None
            }
            _ => {
                // Handle basic text input for variable editing
                match key.code {
                    KeyCode::Char(c) => {
                        // For simplicity, just append to key content
                        state.variable_key_content.push(c);
                    }
                    KeyCode::Backspace => {
                        state.variable_key_content.pop();
                    }
                    _ => {}
                }
                None
            }
        }
    }

    fn handle_template_list_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('l') | KeyCode::Char('L') => {
                state.show_template_list = false;
                None
            }
            KeyCode::Up => {
                if state.template_list_cursor > 0 {
                    state.template_list_cursor -= 1;
                }
                None
            }
            KeyCode::Down => {
                if state.template_list_cursor < state.available_templates.len().saturating_sub(1) {
                    state.template_list_cursor += 1;
                }
                None
            }
            KeyCode::Enter => {
                if let Some(template) = state
                    .available_templates
                    .get(state.template_list_cursor)
                    .cloned()
                {
                    match state.load_template_from_file(&template) {
                        Ok(_) => {
                            state.status_message = format!("Loaded template: {}", template.name);
                        }
                        Err(e) => {
                            state.status_message = e;
                        }
                    }
                }
                state.show_template_list = false;
                None
            }
            _ => None,
        }
    }
}

impl StatefulWidget for TemplateWidget {
    type State = TemplateState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let header_spans = vec![
            Span::styled("Template Editor - ", Style::default().fg(Color::Cyan)),
            Span::styled(
                &state.current_template_name,
                Style::default().fg(Color::White),
            ),
            Span::styled(" | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "l",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("oad | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "s",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("ave | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "r",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "eload | Ctrl+Enter: Run Analysis",
                Style::default().fg(Color::Cyan),
            ),
        ];

        let header = Paragraph::new(Line::from(header_spans))
            .block(Block::default().borders(Borders::ALL).title("Template"))
            .style(Style::default().fg(Color::Cyan));
        header.render(chunks[0], buf);

        // Tabs
        let tab_titles = vec!["Editor", "Variables"];
        let selected_tab = match state.current_tab {
            TemplateTab::Editor => 0,
            TemplateTab::Variables => 1,
        };

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL))
            .select(selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        tabs.render(chunks[1], buf);

        // Content area
        let content_area = chunks[2];

        if state.show_template_list {
            self.render_template_list(content_area, buf, state);
        } else if state.show_variable_editor {
            self.render_variable_editor(content_area, buf, state);
        } else {
            match state.current_tab {
                TemplateTab::Editor => self.render_editor_tab(content_area, buf, state),
                TemplateTab::Variables => self.render_variables_tab(content_area, buf, state),
            }
        }

        // Footer
        self.render_footer(chunks[3], buf, state);
    }
}

impl TemplateWidget {
    fn render_editor_tab(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Use a simple paragraph for template content display
        let content = if state.is_editing {
            format!("EDITING MODE\n\n{}", state.template_content)
        } else {
            state.template_content.clone()
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(if state.is_editing {
                        "Template Content (EDITING)"
                    } else {
                        "Template Content (READ-ONLY)"
                    })
                    .border_style(if state.is_editing {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    }),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        Widget::render(paragraph, area, buf);
    }

    fn render_variables_tab(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Default variables (left side)
        let default_vars = state.get_default_variables();
        let default_items: Vec<ListItem> = default_vars
            .iter()
            .enumerate()
            .map(|(i, (name, desc))| {
                let style = if i == state.variable_list_cursor {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };
                ListItem::new(format!("{{{{ {} }}}}\n  {}", name, desc)).style(style)
            })
            .collect();

        let default_list = List::new(default_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Default Variables"),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        Widget::render(default_list, chunks[0], buf);

        // User variables (right side)
        let user_vars = state.get_user_variables_list();
        let user_items: Vec<ListItem> = user_vars
            .iter()
            .enumerate()
            .map(|(i, (key, value))| {
                let list_index = default_vars.len() + i;
                let style = if list_index == state.variable_list_cursor {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                ListItem::new(format!("{{{{ {} }}}}\n  {}", key, value)).style(style)
            })
            .collect();

        let user_list = List::new(user_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("User Variables"),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        Widget::render(user_list, chunks[1], buf);
    }

    fn render_variable_editor(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let popup_area = Self::centered_rect(60, 40, area);
        Clear.render(popup_area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(popup_area);

        // Simplified variable editor using Paragraph widgets
        let key_content = &state.variable_key_content;
        let value_content = &state.variable_value_content;

        let key_paragraph = Paragraph::new(key_content.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Variable Name"),
        );

        let value_paragraph = Paragraph::new(value_content.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Variable Value"),
        );

        Widget::render(key_paragraph, chunks[0], buf);
        Widget::render(value_paragraph, chunks[1], buf);
    }

    fn render_template_list(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let popup_area = Self::centered_rect(60, 70, area);
        Clear.render(popup_area, buf);

        let items: Vec<ListItem> = state
            .available_templates
            .iter()
            .enumerate()
            .map(|(i, template)| {
                let style = if i == state.template_list_cursor {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if template.is_default {
                    "📄 "
                } else {
                    "📝 "
                };
                ListItem::new(format!("{}{}", prefix, template.name)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Template"),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        Widget::render(list, popup_area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let footer_text = if !state.status_message.is_empty() {
            state.status_message.clone()
        } else {
            match state.current_tab {
                TemplateTab::Editor => {
                    if state.is_editing {
                        "EDIT MODE - Type to edit | e: Exit edit | Tab: Switch tab".to_string()
                    } else {
                        "e: Edit | l: Load | s: Save | r: Reload | Tab: Switch tab".to_string()
                    }
                }
                TemplateTab::Variables => {
                    "↑↓: Navigate | a: Add | Enter: Edit | Del: Delete | Tab: Switch tab"
                        .to_string()
                }
            }
        };

        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        footer.render(area, buf);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
