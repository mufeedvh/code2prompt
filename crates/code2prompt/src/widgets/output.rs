//! Output widget for displaying generated prompt with scrolling capability.

use crate::model::Model;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// State for the output widget - no longer needed, read directly from Model
pub type OutputState = ();

/// Widget for output display with scrolling
pub struct OutputWidget<'a> {
    pub model: &'a Model,
}

impl<'a> OutputWidget<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl<'a> StatefulWidget for OutputWidget<'a> {
    type State = OutputState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Info bar
                Constraint::Min(0),    // Prompt content
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Simplified status bar - focus only on prompt availability
        let info_text = if self.model.prompt_output.analysis_in_progress {
            "Generating prompt...".to_string()
        } else if let Some(error) = &self.model.prompt_output.analysis_error {
            format!("Generation failed: {}", error)
        } else if self.model.prompt_output.generated_prompt.is_some() {
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
            .style(if self.model.prompt_output.analysis_error.is_some() {
                Style::default().fg(Color::Red)
            } else if self.model.prompt_output.analysis_in_progress {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            });
        Widget::render(info_widget, layout[0], buf);

        // Prompt content
        let content = if let Some(prompt) = &self.model.prompt_output.generated_prompt {
            prompt.clone()
        } else if self.model.prompt_output.analysis_in_progress {
            "Generating prompt...".to_string()
        } else {
            "Press <Enter> to run analysis and generate prompt.\n\nSelected files will be processed according to your settings.".to_string()
        };

        // Calculate scroll position for display - read directly from Model
        let scroll_info = if let Some(prompt) = &self.model.prompt_output.generated_prompt {
            let total_lines = prompt.lines().count();
            let current_line = self.model.prompt_output.output_scroll as usize + 1;
            format!("Generated Prompt (Line {}/{})", current_line, total_lines)
        } else {
            "Generated Prompt".to_string()
        };

        let prompt_widget = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(scroll_info))
            .wrap(Wrap { trim: false })
            .scroll((self.model.prompt_output.output_scroll, 0));
        Widget::render(prompt_widget, layout[1], buf);

        // Controls
        let controls_text = if self.model.prompt_output.generated_prompt.is_some() {
            "↑↓/PgUp/PgDn: Scroll | C: Copy | S: Save | Enter: Re-run"
        } else {
            "Enter: Run Analysis"
        };

        let controls_widget = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Gray));
        Widget::render(controls_widget, layout[2], buf);
    }
}
