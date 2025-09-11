//! Template widget for editing and managing Handlebars templates.
//!
//! This widget provides a 3-column interface: template editor, variables manager, and template list.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// State for the template widget
#[derive(Debug, Clone)]
pub struct TemplateState {
    pub template_content: String,
    pub is_editing: bool,
    pub available_templates: Vec<TemplateFile>,
    pub template_list_cursor: usize,
    pub status_message: String,
    pub current_template_name: String,
    pub session_variables: HashMap<String, String>, // Variables from session
    pub user_variables: HashMap<String, String>,    // User-defined variables for current session
    pub template_variables: Vec<String>,            // Variables found in template
    pub missing_variables: Vec<String>,             // Variables in template but not defined
    pub current_column: TemplateColumn,             // Which column is focused
    pub variable_list_cursor: usize,
    pub editing_variable: Option<String>,
    pub variable_input_content: String,
    pub show_variable_input: bool,
}

/// Which column is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateColumn {
    Editor,
    Variables,
    TemplateList,
}

/// Represents a template file
#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
}

/// Variable categories for display
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub value: Option<String>,
    pub category: VariableCategory,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableCategory {
    System,  // From build_template_data
    User,    // User-defined
    Missing, // In template but not defined
}

impl TemplateState {
    pub fn from_model(model: &Model) -> Self {
        let mut state = Self {
            template_content: model.template.template_content.clone(),
            is_editing: false,
            available_templates: Vec::new(),
            template_list_cursor: 0,
            status_message: String::new(),
            current_template_name: "Default".to_string(),
            session_variables: Self::get_session_variables(&model.session.session),
            user_variables: model.session.session.config.user_variables.clone(),
            template_variables: Vec::new(),
            missing_variables: Vec::new(),
            current_column: TemplateColumn::Editor,
            variable_list_cursor: 0,
            editing_variable: None,
            variable_input_content: String::new(),
            show_variable_input: false,
        };

        // Load available templates and analyze current template
        state.load_available_templates();
        state.analyze_template_variables();
        state
    }

    /// Extract system variables that would be available from build_template_data
    fn get_session_variables(
        _session: &code2prompt_core::session::Code2PromptSession,
    ) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // These are the variables available from build_template_data
        vars.insert(
            "absolute_code_path".to_string(),
            "Path to the codebase directory".to_string(),
        );
        vars.insert(
            "source_tree".to_string(),
            "Directory tree structure".to_string(),
        );
        vars.insert(
            "files".to_string(),
            "Array of file objects with content".to_string(),
        );
        vars.insert(
            "git_diff".to_string(),
            "Git diff output (if enabled)".to_string(),
        );
        vars.insert(
            "git_diff_branch".to_string(),
            "Git diff between branches".to_string(),
        );
        vars.insert(
            "git_log_branch".to_string(),
            "Git log between branches".to_string(),
        );

        vars
    }

    /// Parse template content to extract all {{variable}} references
    fn analyze_template_variables(&mut self) {
        // Use regex to find all {{variable}} patterns
        let re = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
        let mut found_vars = HashSet::new();

        for cap in re.captures_iter(&self.template_content) {
            if let Some(var_name) = cap.get(1) {
                found_vars.insert(var_name.as_str().to_string());
            }
        }

        self.template_variables = found_vars.into_iter().collect();
        self.template_variables.sort();

        // Find missing variables (in template but not defined)
        self.missing_variables.clear();
        for var in &self.template_variables {
            if !self.session_variables.contains_key(var) && !self.user_variables.contains_key(var) {
                self.missing_variables.push(var.clone());
            }
        }
    }

    /// Get all variables organized by category
    pub fn get_organized_variables(&self) -> Vec<VariableInfo> {
        let mut variables = Vec::new();

        // System variables (from session)
        for var in &self.template_variables {
            if let Some(desc) = self.session_variables.get(var) {
                variables.push(VariableInfo {
                    name: var.clone(),
                    value: Some("(system)".to_string()),
                    category: VariableCategory::System,
                    description: Some(desc.clone()),
                });
            }
        }

        // User variables
        for var in &self.template_variables {
            if let Some(value) = self.user_variables.get(var) {
                variables.push(VariableInfo {
                    name: var.clone(),
                    value: Some(value.clone()),
                    category: VariableCategory::User,
                    description: None,
                });
            }
        }

        // Missing variables
        for var in &self.missing_variables {
            variables.push(VariableInfo {
                name: var.clone(),
                value: None,
                category: VariableCategory::Missing,
                description: Some("⚠️ Not defined".to_string()),
            });
        }

        variables
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
        self.analyze_template_variables();
        Ok(())
    }

    pub fn set_user_variable(&mut self, key: String, value: String) {
        self.user_variables.insert(key, value);
        self.analyze_template_variables(); // Re-analyze to update missing variables
    }

    pub fn has_missing_variables(&self) -> bool {
        !self.missing_variables.is_empty()
    }

    pub fn get_missing_variables_message(&self) -> String {
        if self.missing_variables.is_empty() {
            String::new()
        } else {
            format!(
                "Missing variables: {}. Please define them in the Variables column.",
                self.missing_variables.join(", ")
            )
        }
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
        if state.show_variable_input {
            return Self::handle_variable_input_keys(key, state);
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('l') | KeyCode::Char('L') => {
                state.current_column = TemplateColumn::TemplateList;
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
            KeyCode::Tab => {
                // Cycle through columns
                state.current_column = match state.current_column {
                    TemplateColumn::Editor => TemplateColumn::Variables,
                    TemplateColumn::Variables => TemplateColumn::TemplateList,
                    TemplateColumn::TemplateList => TemplateColumn::Editor,
                };
                None
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Check for missing variables before running analysis
                if state.has_missing_variables() {
                    state.status_message = state.get_missing_variables_message();
                    state.current_column = TemplateColumn::Variables;
                    None
                } else {
                    Some(Message::RunAnalysis)
                }
            }
            _ => match state.current_column {
                TemplateColumn::Editor => Self::handle_editor_keys(key, state),
                TemplateColumn::Variables => Self::handle_variables_keys(key, state),
                TemplateColumn::TemplateList => Self::handle_template_list_keys(key, state),
            },
        }
    }

    fn handle_editor_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Char('e') | KeyCode::Char('E') => {
                state.is_editing = !state.is_editing;
                if !state.is_editing {
                    // Re-analyze template when exiting edit mode
                    state.analyze_template_variables();
                }
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
        let variables = state.get_organized_variables();

        match key.code {
            KeyCode::Up => {
                if state.variable_list_cursor > 0 {
                    state.variable_list_cursor -= 1;
                }
                None
            }
            KeyCode::Down => {
                if state.variable_list_cursor < variables.len().saturating_sub(1) {
                    state.variable_list_cursor += 1;
                }
                None
            }
            KeyCode::Enter => {
                // Edit variable if it's user-defined or missing
                if let Some(var_info) = variables.get(state.variable_list_cursor) {
                    match var_info.category {
                        VariableCategory::User | VariableCategory::Missing => {
                            state.editing_variable = Some(var_info.name.clone());
                            state.variable_input_content =
                                var_info.value.clone().unwrap_or_default();
                            state.show_variable_input = true;
                        }
                        VariableCategory::System => {
                            state.status_message = "System variables cannot be edited".to_string();
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn handle_template_list_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
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
                            state.current_column = TemplateColumn::Editor;
                        }
                        Err(e) => {
                            state.status_message = e;
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn handle_variable_input_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Esc => {
                state.show_variable_input = false;
                state.editing_variable = None;
                state.variable_input_content.clear();
                None
            }
            KeyCode::Enter => {
                // Save variable
                if let Some(var_name) = state.editing_variable.clone() {
                    let var_value = state.variable_input_content.clone();
                    state.set_user_variable(var_name.clone(), var_value.clone());
                    state.status_message = format!("Set {} = {}", var_name, var_value);
                    state.show_variable_input = false;
                    state.editing_variable = None;
                    state.variable_input_content.clear();
                }
                None
            }
            KeyCode::Char(c) => {
                state.variable_input_content.push(c);
                None
            }
            KeyCode::Backspace => {
                state.variable_input_content.pop();
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
                Constraint::Length(3), // Header with Controls
                Constraint::Min(0),    // Content (3 columns)
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header with Controls
        self.render_header(chunks[0], buf, state);

        // 3-column layout for content
        self.render_content(chunks[1], buf, state);

        // Footer
        self.render_footer(chunks[2], buf, state);

        // Variable input popup if active
        if state.show_variable_input {
            self.render_variable_input(area, buf, state);
        }
    }
}

impl TemplateWidget {
    fn render_header(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let header_spans = vec![
            Span::styled("Template Editor - ", Style::default().fg(Color::Cyan)),
            Span::styled(
                &state.current_template_name,
                Style::default().fg(Color::White),
            ),
            Span::styled(" | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "L",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("oad | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "S",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("ave | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "R",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "eload | Ctrl+Enter: Run Analysis",
                Style::default().fg(Color::Cyan),
            ),
        ];

        let header = Paragraph::new(Line::from(header_spans))
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Cyan));
        header.render(area, buf);
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Flexible 3-column layout
        let min_width = 30;
        let available_width = area.width.saturating_sub(6); // Account for borders

        let constraints = if available_width >= min_width * 3 {
            // Full 3-column layout
            vec![
                Constraint::Percentage(40), // Editor
                Constraint::Percentage(35), // Variables
                Constraint::Percentage(25), // Template list
            ]
        } else if available_width >= min_width * 2 {
            // 2-column layout, hide template list or make it smaller
            vec![
                Constraint::Percentage(60), // Editor
                Constraint::Percentage(40), // Variables
                Constraint::Length(0),      // Template list hidden
            ]
        } else {
            // Single column, show only focused column
            match state.current_column {
                TemplateColumn::Editor => vec![
                    Constraint::Percentage(100),
                    Constraint::Length(0),
                    Constraint::Length(0),
                ],
                TemplateColumn::Variables => vec![
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                    Constraint::Length(0),
                ],
                TemplateColumn::TemplateList => vec![
                    Constraint::Length(0),
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                ],
            }
        };

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Render each column if it has space
        if columns[0].width > 0 {
            self.render_editor_column(columns[0], buf, state);
        }
        if columns[1].width > 0 {
            self.render_variables_column(columns[1], buf, state);
        }
        if columns[2].width > 0 {
            self.render_template_list_column(columns[2], buf, state);
        }
    }

    fn render_editor_column(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let is_focused = state.current_column == TemplateColumn::Editor;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let content = if state.is_editing {
            format!("EDITING MODE\n\n{}", state.template_content)
        } else {
            state.template_content.clone()
        };

        let title = if state.is_editing {
            "Template Editor (EDITING)"
        } else {
            "Template Editor (e: edit)"
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        Widget::render(paragraph, area, buf);
    }

    fn render_variables_column(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let is_focused = state.current_column == TemplateColumn::Variables;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let variables = state.get_organized_variables();
        let items: Vec<ListItem> = variables
            .iter()
            .enumerate()
            .map(|(i, var_info)| {
                let style = if i == state.variable_list_cursor && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    match var_info.category {
                        VariableCategory::System => Style::default().fg(Color::Green),
                        VariableCategory::User => Style::default().fg(Color::Cyan),
                        VariableCategory::Missing => Style::default().fg(Color::Red),
                    }
                };

                let display_value = match &var_info.value {
                    Some(val) => val.clone(),
                    None => "❌ undefined".to_string(),
                };

                let text = format!(
                    "{{{{ {} }}}}\n  {}",
                    var_info.name,
                    var_info.description.as_ref().unwrap_or(&display_value)
                );

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Variables (Enter: edit)")
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        Widget::render(list, area, buf);
    }

    fn render_template_list_column(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let is_focused = state.current_column == TemplateColumn::TemplateList;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<ListItem> = state
            .available_templates
            .iter()
            .enumerate()
            .map(|(i, template)| {
                let style = if i == state.template_list_cursor && is_focused {
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
                    .title("Templates (Enter: load)")
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        Widget::render(list, area, buf);
    }

    fn render_variable_input(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let popup_area = Self::centered_rect(60, 20, area);
        Clear.render(popup_area, buf);

        let default_name = String::new();
        let var_name = state.editing_variable.as_ref().unwrap_or(&default_name);
        let title = format!("Set Variable: {}", var_name);

        let paragraph = Paragraph::new(state.variable_input_content.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        Widget::render(paragraph, popup_area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let footer_text = if !state.status_message.is_empty() {
            state.status_message.clone()
        } else {
            match state.current_column {
                TemplateColumn::Editor => {
                    if state.is_editing {
                        "EDIT MODE - Type to edit | e: Exit edit | Tab: Switch column".to_string()
                    } else {
                        "e: Edit | Tab: Switch column | Ctrl+Enter: Run Analysis".to_string()
                    }
                }
                TemplateColumn::Variables => {
                    "↑↓: Navigate | Enter: Edit variable | Tab: Switch column".to_string()
                }
                TemplateColumn::TemplateList => {
                    "↑↓: Navigate | Enter: Load template | Tab: Switch column".to_string()
                }
            }
        };

        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
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
