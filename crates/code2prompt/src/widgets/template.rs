//! Template widget for editing and managing Handlebars templates.
//!
//! This widget provides a text editor interface for modifying templates,
//! loading templates from files, and saving custom templates.

use crate::model::{Message, Model};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::fs;
use std::path::PathBuf;

/// State for the template widget
#[derive(Debug, Clone)]
pub struct TemplateState {
    pub template_content: String,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub is_editing: bool,
    pub available_templates: Vec<TemplateFile>,
    pub template_list_cursor: usize,
    pub show_template_list: bool,
    pub status_message: String,
    pub current_template_name: String,
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
            cursor_position: 0,
            scroll_offset: 0,
            is_editing: false,
            available_templates: Vec::new(),
            template_list_cursor: 0,
            show_template_list: false,
            status_message: String::new(),
            current_template_name: "Default".to_string(),
        };

        // Load available templates from directory
        state.load_available_templates();
        state
    }

    fn load_default_template(&mut self, model: &Model) {
        match model.session.session.config.output_format {
            code2prompt_core::template::OutputFormat::Markdown => {
                self.template_content =
                    include_str!("../../../code2prompt-core/src/default_template_md.hbs")
                        .to_string();
                self.current_template_name = "Default (Markdown)".to_string();
            }
            code2prompt_core::template::OutputFormat::Xml
            | code2prompt_core::template::OutputFormat::Json => {
                self.template_content =
                    include_str!("../../../code2prompt-core/src/default_template_xml.hbs")
                        .to_string();
                self.current_template_name = "Default (XML)".to_string();
            }
        }
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

        // Load templates from the templates directory
        let templates_dir = PathBuf::from("crates/code2prompt-core/templates");
        if templates_dir.exists() {
            if let Ok(entries) = fs::read_dir(&templates_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            self.available_templates.push(TemplateFile {
                                name: name.replace('-', " ").replace('_', " "),
                                path: path.clone(),
                                is_default: false,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn load_template_from_file(&mut self, template_file: &TemplateFile) -> Result<(), String> {
        if template_file.is_default {
            match template_file.name.as_str() {
                "Default (Markdown)" => {
                    self.template_content =
                        include_str!("../../../code2prompt-core/src/default_template_md.hbs")
                            .to_string();
                }
                "Default (XML)" => {
                    self.template_content =
                        include_str!("../../../code2prompt-core/src/default_template_xml.hbs")
                            .to_string();
                }
                _ => return Err("Unknown default template".to_string()),
            }
        } else {
            match fs::read_to_string(&template_file.path) {
                Ok(content) => {
                    self.template_content = content;
                }
                Err(e) => return Err(format!("Failed to load template: {}", e)),
            }
        }

        self.current_template_name = template_file.name.clone();
        self.cursor_position = 0;
        self.scroll_offset = 0;
        Ok(())
    }

    pub fn save_template_to_file(&self, filename: &str) -> Result<(), String> {
        let templates_dir = PathBuf::from("crates/code2prompt-core/templates");
        if !templates_dir.exists() {
            if let Err(e) = fs::create_dir_all(&templates_dir) {
                return Err(format!("Failed to create templates directory: {}", e));
            }
        }

        let file_path = templates_dir.join(format!("{}.hbs", filename));
        match fs::write(&file_path, &self.template_content) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to save template: {}", e)),
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
        model: &Model,
        state: &mut TemplateState,
    ) -> Option<Message> {
        if state.show_template_list {
            return Self::handle_template_list_keys(key, state);
        }

        match key.code {
            KeyCode::Char('l') | KeyCode::Char('L') => {
                state.show_template_list = true;
                None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Save template dialog - for now just save with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("custom_template_{}", timestamp);
                match state.save_template_to_file(&filename) {
                    Ok(_) => {
                        state.status_message = format!("Template saved as {}.hbs", filename);
                        state.load_available_templates();
                    }
                    Err(e) => {
                        state.status_message = format!("Save failed: {}", e);
                    }
                }
                None
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reload current template
                state.load_default_template(model);
                state.status_message = "Template reloaded".to_string();
                None
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Run analysis with current template
                Some(Message::RunAnalysis)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                state.is_editing = !state.is_editing;
                state.status_message = if state.is_editing {
                    "Edit mode enabled - Use arrow keys and type to edit".to_string()
                } else {
                    "Edit mode disabled".to_string()
                };
                None
            }
            _ if state.is_editing => Self::handle_edit_keys(key, state),
            KeyCode::Up => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
                None
            }
            KeyCode::Down => {
                state.scroll_offset += 1;
                None
            }
            KeyCode::PageUp => {
                state.scroll_offset = state.scroll_offset.saturating_sub(10);
                None
            }
            KeyCode::PageDown => {
                state.scroll_offset += 10;
                None
            }
            _ => None,
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

    fn handle_edit_keys(key: KeyEvent, state: &mut TemplateState) -> Option<Message> {
        match key.code {
            KeyCode::Char(c) => {
                state.template_content.insert(state.cursor_position, c);
                state.cursor_position += 1;
                None
            }
            KeyCode::Backspace => {
                if state.cursor_position > 0 {
                    state.cursor_position -= 1;
                    state.template_content.remove(state.cursor_position);
                }
                None
            }
            KeyCode::Delete => {
                if state.cursor_position < state.template_content.len() {
                    state.template_content.remove(state.cursor_position);
                }
                None
            }
            KeyCode::Left => {
                if state.cursor_position > 0 {
                    state.cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if state.cursor_position < state.template_content.len() {
                    state.cursor_position += 1;
                }
                None
            }
            KeyCode::Home => {
                // Move to beginning of current line
                let lines: Vec<&str> = state.template_content[..state.cursor_position]
                    .split('\n')
                    .collect();
                if let Some(current_line) = lines.last() {
                    state.cursor_position -= current_line.len();
                }
                None
            }
            KeyCode::End => {
                // Move to end of current line
                let remaining = &state.template_content[state.cursor_position..];
                if let Some(newline_pos) = remaining.find('\n') {
                    state.cursor_position += newline_pos;
                } else {
                    state.cursor_position = state.template_content.len();
                }
                None
            }
            KeyCode::Enter => {
                state.template_content.insert(state.cursor_position, '\n');
                state.cursor_position += 1;
                None
            }
            _ => None,
        }
    }
}

impl StatefulWidget for TemplateWidget {
    type State = TemplateState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Handle keyboard input if we have access to events
        // This is a workaround since StatefulWidget doesn't provide event handling

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header with red visual indicators for keyboard shortcuts
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
            Span::styled("eload | ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "e",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if state.is_editing {
                    "dit: Exit | "
                } else {
                    "dit | "
                },
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("Ctrl+Enter: Run Analysis", Style::default().fg(Color::Cyan)),
        ];

        let header = Paragraph::new(Line::from(header_spans))
            .block(Block::default().borders(Borders::ALL).title("Template"))
            .style(Style::default().fg(Color::Cyan));
        header.render(chunks[0], buf);

        // Content area
        let content_area = chunks[1];

        if state.show_template_list {
            // Show template selection overlay
            self.render_template_list(content_area, buf, state);
        } else {
            // Show template editor
            self.render_template_editor(content_area, buf, state);
        }

        // Footer with status and red visual indicators
        if !state.status_message.is_empty() {
            let footer = Paragraph::new(state.status_message.clone())
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Green));
            footer.render(chunks[2], buf);
        } else if state.is_editing {
            let footer = Paragraph::new("EDIT MODE - Type to edit template | Esc: Exit edit mode")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow));
            footer.render(chunks[2], buf);
        } else {
            let footer_spans = vec![
                Span::styled("Arrow keys: Scroll | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("dit | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "l",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("oad | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "s",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("ave | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "r",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "eload | Ctrl+Enter: Run analysis",
                    Style::default().fg(Color::Gray),
                ),
            ];

            let footer = Paragraph::new(Line::from(footer_spans))
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));
            footer.render(chunks[2], buf);
        }
    }
}

impl TemplateWidget {
    fn render_template_editor(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        let lines: Vec<&str> = state.template_content.lines().collect();
        let visible_lines: Vec<String> = lines
            .iter()
            .skip(state.scroll_offset as usize)
            .take(area.height as usize)
            .enumerate()
            .map(|(i, line)| {
                let line_num = state.scroll_offset as usize + i + 1;
                if state.is_editing {
                    // Show cursor position if we're in edit mode
                    let cursor_line = state.template_content[..state.cursor_position]
                        .lines()
                        .count();
                    if cursor_line == line_num {
                        format!("► {:3} | {}", line_num, line)
                    } else {
                        format!("  {:3} | {}", line_num, line)
                    }
                } else {
                    format!("  {:3} | {}", line_num, line)
                }
            })
            .collect();

        let content = visible_lines.join("\n");
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(if state.is_editing {
                        "Template Content (EDITING)"
                    } else {
                        "Template Content (READ-ONLY)"
                    }),
            )
            .wrap(Wrap { trim: false })
            .style(if state.is_editing {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            });

        paragraph.render(area, buf);
    }

    fn render_template_list(&self, area: Rect, buf: &mut Buffer, state: &mut TemplateState) {
        // Create a centered popup
        let popup_area = Self::centered_rect(60, 70, area);

        // Clear the background
        Clear.render(popup_area, buf);

        // Create the list items
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
