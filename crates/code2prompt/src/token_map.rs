//! Token map visualization (View Layer).
//!
//! This module transforms the raw topological data from `analysis` into ASCII art.
//! It owns all presentation logic: colors, indentation guides, and formatting.

use crate::utils::format_number;
use code2prompt_core::analysis::TokenMapEntry;
use code2prompt_core::tokenizer::TokenFormat;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// Color information for TUI rendering
#[derive(Debug, Clone, Copy)]
pub enum TuiColor {
    White,
    Gray,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    LightRed,
    LightGreen,
    LightBlue,
    LightYellow,
    LightCyan,
    LightMagenta,
}

/// Formatted line for TUI/CLI display
#[derive(Debug, Clone)]
pub struct TuiTokenMapLine {
    pub tokens_part: String,
    pub prefix_part: String,
    pub name_part: String,
    pub name_color: TuiColor,
    pub bar_part: String,
    pub percentage_part: String,
}

/// Display a visual token map in the CLI.
pub fn display_token_map(entries: &[TokenMapEntry], total_tokens: usize) {
    if entries.is_empty() {
        return;
    }

    let terminal_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(80);

    let formatted_lines =
        format_token_map_for_tui(entries, total_tokens, terminal_width, &TokenFormat::Format);

    for line in formatted_lines {
        let colored_name = apply_cli_color(&line.name_part, line.name_color);
        eprintln!(
            "{}   {}{} │{}│ {}",
            line.tokens_part, line.prefix_part, colored_name, line.bar_part, line.percentage_part
        );
    }
}

/// Main formatting engine.
/// Used by both CLI (display_token_map) and TUI widgets.
pub fn format_token_map_for_tui(
    entries: &[TokenMapEntry],
    total_tokens: usize,
    terminal_width: usize,
    token_format: &TokenFormat,
) -> Vec<TuiTokenMapLine> {
    if entries.is_empty() {
        return Vec::new();
    }

    let terminal_width = terminal_width.max(80);

    // 1. Calculate Layout Constraints
    let max_token_width = entries
        .iter()
        .map(|e| format_number(e.tokens, token_format).len())
        .max()
        .unwrap_or(3)
        .max(format_number(total_tokens, token_format).len())
        .max(4);

    let max_depth = entries.iter().map(|e| e.depth).max().unwrap_or(0);

    // Calculate max name length considering the tree prefix width
    let max_name_length = entries
        .iter()
        .map(|e| {
            let prefix_len = (e.depth * 2) + 3; // "│ " * depth + "├─ "
            prefix_len + UnicodeWidthStr::width(e.name.as_str())
        })
        .max()
        .unwrap_or(20)
        .min(terminal_width / 2);

    let bar_width = terminal_width
        .saturating_sub(max_token_width + 3 + max_name_length + 2 + 2 + 7)
        .max(15);

    // 2. Rendering Loop
    let mut parent_bars: Vec<String> = vec![String::new(); max_depth + 2];
    parent_bars[0] = "█".repeat(bar_width);

    // State for tree guides: is the ancestor at depth `i` NOT last?
    // true = draw "│", false = draw " "
    let mut open_branches: Vec<bool> = Vec::new();

    let mut lines = Vec::new();

    for entry in entries {
        // -- Logic: Tree Prefix Generation --
        // Update guide state based on depth
        if entry.depth < open_branches.len() {
            open_branches.truncate(entry.depth);
        }

        let mut prefix = String::new();
        // Draw ancestors
        for &is_open in &open_branches {
            prefix.push_str(if is_open { "│ " } else { "  " });
        }
        // Draw current connector
        prefix.push_str(if entry.is_last_child {
            "└─"
        } else {
            "├─"
        });
        // Draw branch indicator
        prefix.push_str(if entry.has_children { "┬ " } else { "─ " });

        // Update state for children
        open_branches.push(!entry.is_last_child);
        // -----------------------------------

        // Format Numbers
        let tokens_str = format_number(entry.tokens, token_format);
        let percentage_str = format!("{:>4.0}%", entry.percentage);

        // -- Logic: Bar Generation --
        let parent_bar = if entry.depth > 0 {
            &parent_bars[entry.depth - 1]
        } else {
            &parent_bars[0]
        };
        let bar = generate_hierarchical_bar(bar_width, parent_bar, entry.percentage, entry.depth);
        if entry.depth < parent_bars.len() {
            parent_bars[entry.depth] = bar.clone();
        }
        // ----------------------------

        // Name alignment
        let prefix_display_width = (entry.depth * 2) + 3;
        let name_padding = max_name_length
            .saturating_sub(prefix_display_width + UnicodeWidthStr::width(entry.name.as_str()));
        let name_with_padding = format!("{}{}", entry.name, " ".repeat(name_padding));

        lines.push(TuiTokenMapLine {
            tokens_part: format!("{:>width$}", tokens_str, width = max_token_width),
            prefix_part: prefix,
            name_part: name_with_padding,
            name_color: determine_tui_color(entry),
            bar_part: bar,
            percentage_part: percentage_str,
        });
    }

    lines
}

/// Helper: Color logic (same as before, just mapped cleanly)
fn determine_tui_color(entry: &TokenMapEntry) -> TuiColor {
    if entry.name == "(other files)" {
        return TuiColor::Gray;
    }
    if entry.metadata.is_dir {
        return TuiColor::Cyan;
    }

    match entry.name.split('.').next_back().unwrap_or("") {
        "rs" => TuiColor::Yellow,
        "js" | "ts" | "jsx" | "tsx" => TuiColor::LightGreen,
        "py" => TuiColor::LightYellow,
        "go" => TuiColor::LightBlue,
        "md" | "txt" => TuiColor::Green,
        "json" | "toml" | "yaml" => TuiColor::Magenta,
        _ => TuiColor::White,
    }
}

fn apply_cli_color(text: &str, color: TuiColor) -> colored::ColoredString {
    match color {
        TuiColor::White => text.white(),
        TuiColor::Gray => text.truecolor(128, 128, 128),
        TuiColor::Red => text.red(),
        TuiColor::Green => text.green(),
        TuiColor::Blue => text.blue(),
        TuiColor::Yellow => text.yellow(),
        TuiColor::Cyan => text.cyan(),
        TuiColor::Magenta => text.magenta(),
        TuiColor::LightRed => text.bright_red(),
        TuiColor::LightGreen => text.bright_green(),
        TuiColor::LightBlue => text.bright_blue(),
        TuiColor::LightYellow => text.bright_yellow(),
        TuiColor::LightCyan => text.bright_cyan(),
        TuiColor::LightMagenta => text.bright_magenta(),
    }
}

fn generate_hierarchical_bar(
    bar_width: usize,
    parent_bar: &str,
    percentage: f64,
    depth: usize,
) -> String {
    let filled_chars = ((percentage / 100.0) * bar_width as f64).round() as usize;
    let mut result = String::new();
    let shade_char = match depth {
        0 => '█',
        1 => '░',
        2 => '▒',
        _ => '▓',
    };

    let parent_chars: Vec<char> = parent_bar.chars().collect();
    for i in 0..bar_width {
        if i < filled_chars {
            result.push('█'); // Self is always solid
        } else if i < parent_chars.len() {
            // Parent ghosting
            result.push(if parent_chars[i] == '█' {
                shade_char
            } else {
                parent_chars[i]
            });
        } else {
            result.push(' ');
        }
    }
    result
}
