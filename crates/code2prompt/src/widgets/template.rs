//! Template widget for editing and managing Handlebars templates.
//!
//! This widget provides a 3-column interface: template editor, variables manager, and template list.

use crate::model::{Message, Model};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tui_textarea::TextArea;

/// State for the template widget
#[derive(Debug)]
pub struct TemplateState {
    pub template_content: String,
    pub template_editor: TextArea<'static>, // TextArea for template editing
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
    pub show_save_dialog: bool,        // Show save template dialog
    pub save_filename_input: String,   // Input for save filename
    pub editor_cursor_position: usize, // Cursor position in editor
    pub editor_scroll_offset: usize,   // Scroll offset for editor
}

// Remove Clone derive since TextArea doesn't implement Clone
impl Clone for TemplateState {
    fn clone(&self) -> Self {
        let mut new_editor =
            TextArea::from(self.template_editor.lines().iter().map(|s| s.as_str()));
        new_editor.move_cursor(tui_textarea::CursorMove::Jump(
            self.template_editor.cursor().0.try_into().unwrap_or(0),
            self.template_editor.cursor().1.try_into().unwrap_or(0),
        ));

        Self {
            template_content: self.template_content.clone(),
            template_editor: new_editor,
            is_editing: self.is_editing,
            available_templates: self.available_templates.clone(),
            template_list_cursor: self.template_list_cursor,
            status_message: self.status_message.clone(),
            current_template_name: self.current_template_name.clone(),
            session_variables: self.session_variables.clone(),
            user_variables: self.user_variables.clone(),
            template_variables: self.template_variables.clone(),
            missing_variables: self.missing_variables.clone(),
            current_column: self.current_column,
            variable_list_cursor: self.variable_list_cursor,
            editing_variable: self.editing_variable.clone(),
            variable_input_content: self.variable_input_content.clone(),
            show_variable_input: self.show_variable_input,
            show_save_dialog: self.show_save_dialog,
            save_filename_input: self.save_filename_input.clone(),
            editor_cursor_position: self.editor_cursor_position,
            editor_scroll_offset: self.editor_scroll_offset,
        }
    }
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
        // Create TextArea from template content
        let template_editor = TextArea::from(model.template.template_content.lines());

        let mut state = Self {
            template_content: model.template.template_content.clone(),
            template_editor,
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
            show_save_dialog: false,
            save_filename_input: String::new(),
            editor_cursor_position: 0,
            editor_scroll_offset: 0,
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

        // Load all templates using the new utility function
        if let Ok(all_templates) = crate::utils::load_all_templates() {
            for (name, path, _is_builtin) in all_templates {
                self.available_templates.push(TemplateFile {
                    name,
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

        // Update TextArea with new content
        self.template_editor = TextArea::from(content.lines());

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

        // Global shortcuts - Focus system (e/v/p)
        match key.code {
            KeyCode::Char('e') | KeyCode::Char('E') if !state.is_editing => {
                state.current_column = TemplateColumn::Editor;
                None
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                state.current_column = TemplateColumn::Variables;
                None
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                state.current_column = TemplateColumn::TemplateList;
                None
            }
            KeyCode::Char('l') | KeyCode::Char('L')
                if state.current_column == TemplateColumn::TemplateList =>
            {
                // Load template only when in template list column
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
                if state.is_editing {
                    // Sync TextArea with current content when entering edit mode
                    state.template_editor = TextArea::from(state.template_content.lines());
                } else {
                    // Sync content from TextArea when exiting edit mode
                    state.template_content = state.template_editor.lines().join("\n");
                    state.analyze_template_variables();
                }
                Some(Message::ToggleTemplateEdit)
            }
            _ if state.is_editing => {
                // Use TextArea's input handling when in edit mode
                state.template_editor.input(key);
                // Keep template_content in sync
                state.template_content = state.template_editor.lines().join("\n");
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
        // Main layout - Remove header, keep only content and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Content (3 columns)
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // 3-column layout for content
        self.render_content(chunks[0], buf, state);

        // Footer
        self.render_footer(chunks[1], buf, state);

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
            Span::styled("eload", Style::default().fg(Color::Cyan)),
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

        let title_spans = if state.is_editing {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "E",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ditor (EDITING)", Style::default().fg(Color::White)),
            ]
        } else {
            vec![
                Span::styled("Template ", Style::default().fg(Color::White)),
                Span::styled(
                    "E",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ditor", Style::default().fg(Color::White)),
            ]
        };

        if state.is_editing {
            // Use TextArea for editing mode
            let mut textarea = state.template_editor.clone();
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans))
                    .border_style(border_style),
            );

            // Set cursor line style if focused
            if is_focused {
                textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            }

            Widget::render(&textarea, area, buf);
        } else {
            // Use Paragraph for read-only mode
            let paragraph = Paragraph::new(state.template_content.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(title_spans))
                        .border_style(border_style),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });

            Widget::render(paragraph, area, buf);
        }
    }

    fn render_variables_column(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let is_focused = state.current_column == TemplateColumn::Variables;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let variables = state.get_organized_variables();

        // Create table-like display with 2 columns
        let mut lines = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::styled(
                "Name",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("                    "), // Spacing
            Span::styled(
                "Description/Value",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![Span::raw(
            "────────────────────────────────────────────────────────────────────────────────",
        )]));

        // Variable rows
        for (i, var_info) in variables.iter().enumerate() {
            let is_selected = i == state.variable_list_cursor && is_focused;

            let name_style = if is_selected {
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

            let value_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = match var_info.category {
                VariableCategory::System => "🔧 ",
                VariableCategory::User => "👤 ",
                VariableCategory::Missing => "❌ ",
            };

            let name_part = format!("{}{{{{{}}}}}", prefix, var_info.name);
            let name_padded = format!("{:<20}", name_part);

            let value_part = match var_info.category {
                VariableCategory::System => var_info
                    .description
                    .as_ref()
                    .unwrap_or(&"System variable".to_string())
                    .clone(),
                VariableCategory::User => var_info
                    .value
                    .as_ref()
                    .unwrap_or(&"(empty)".to_string())
                    .clone(),
                VariableCategory::Missing => "⚠️ Not defined - Press Enter to set".to_string(),
            };

            let line = if is_selected {
                // Highlight entire row for selected item
                Line::from(vec![Span::styled(
                    format!("► {}{}", name_padded, value_part),
                    name_style,
                )])
            } else {
                Line::from(vec![
                    Span::styled(format!("  {}", name_padded), name_style),
                    Span::styled(value_part, value_style),
                ])
            };

            lines.push(line);
        }

        let title_spans = vec![
            Span::styled(
                "V",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("ariables", Style::default().fg(Color::White)),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans))
                    .border_style(border_style),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        Widget::render(paragraph, area, buf);
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

        let title_spans = vec![
            Span::styled("Template ", Style::default().fg(Color::White)),
            Span::styled(
                "P",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("icker (Enter/L: load)", Style::default().fg(Color::White)),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(title_spans))
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
            // Global controls with focus system
            let global_controls = "Focus: E(dit) V(ariables) P(icker) | S(ave) R(eload)";

            let specific_controls = match state.current_column {
                TemplateColumn::Editor => {
                    if state.is_editing {
                        " | EDIT MODE: Type to edit, E to exit"
                    } else {
                        " | E: Edit template, ↑↓: Scroll"
                    }
                }
                TemplateColumn::Variables => " | ↑↓: Navigate, Enter: Edit variable",
                TemplateColumn::TemplateList => " | ↑↓: Navigate, Enter/L: Load template",
            };

            format!("{}{}", global_controls, specific_controls)
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
