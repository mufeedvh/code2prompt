//! Token map visualization (View Layer).
//!
//! This module provides rendering functionality for displaying token maps
//! in both CLI and TUI modes. It unifies the logic so CLI looks exactly like TUI.

use crate::utils::format_number;
use code2prompt_core::analysis::TokenMapEntry;
use code2prompt_core::tokenizer::TokenFormat;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// Color information for TUI rendering (Intermediate representation)
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

/// Formatted line for TUI/CLI display with separate components
#[derive(Debug, Clone)]
pub struct TuiTokenMapLine {
    pub tokens_part: String,
    pub prefix_part: String,
    pub name_part: String,
    pub name_color: TuiColor,
    pub bar_part: String,
    pub percentage_part: String,
}

/// Display a visual token map in the CLI using the shared TUI formatting logic.
///
/// This ensures strict visual parity between CLI and TUI.
pub fn display_token_map(entries: &[TokenMapEntry], total_tokens: usize) {
    if entries.is_empty() {
        return;
    }

    // Detect terminal width
    let terminal_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(80);

    // 1. REUSE: Call the shared formatting logic used by the TUI
    let formatted_lines =
        format_token_map_for_tui(entries, total_tokens, terminal_width, &TokenFormat::Format);

    // 2. RENDER: Print lines to stderr using 'colored' crate for ANSI output
    for line in formatted_lines {
        let colored_name = apply_cli_color(&line.name_part, line.name_color);

        // Output format: TOKENS | PREFIX | NAME | BAR | PERCENTAGE
        eprintln!(
            "{}   {}{} │{}│ {}",
            line.tokens_part, line.prefix_part, colored_name, line.bar_part, line.percentage_part
        );
    }
}

/// Helper to convert internal TuiColor to 'colored' crate styles for CLI
fn apply_cli_color(text: &str, color: TuiColor) -> colored::ColoredString {
    match color {
        TuiColor::White => text.white(),
        TuiColor::Gray => text.truecolor(128, 128, 128), // Gray approximation
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

/// Build tree prefix for an entry
fn build_tree_prefix(entry: &TokenMapEntry, entries: &[TokenMapEntry], index: usize) -> String {
    let mut prefix = String::new();

    // Add vertical lines for parent levels
    for d in 0..entry.depth {
        if d < entry.depth - 1 {
            // Check if we need a vertical line at this depth
            let needs_line = entries
                .iter()
                .skip(index + 1)
                .take_while(|e| e.depth > d)
                .any(|e| e.depth == d + 1);
            if needs_line {
                prefix.push_str("│ ");
            } else {
                prefix.push_str("  ");
            }
        } else if entry.is_last {
            prefix.push_str("└─");
        } else {
            prefix.push_str("├─");
        }
    }

    // Special handling for root (depth 0)
    if entry.depth == 0 {
        if index == 0 && entries.len() > 1 {
            prefix = "┌─".to_string();
        } else if index == entries.len() - 1 && index > 0 {
            prefix = "└─".to_string();
        } else if entries.len() > 1 {
            prefix = "├─".to_string();
        } else {
            prefix = "──".to_string(); // Single item list
        }
    }

    // Check if has children (next item is deeper)
    let has_children = entries
        .get(index + 1)
        .map(|next| next.depth > entry.depth)
        .unwrap_or(false);

    // Add the connecting character
    if has_children {
        prefix.push('┬');
    } else {
        prefix.push('─');
    }

    prefix.push(' ');
    prefix
}

/// Determine TUI color for an entry based on file type and extension
fn determine_tui_color(entry: &TokenMapEntry) -> TuiColor {
    if entry.name == "(other files)" {
        return TuiColor::Gray;
    }

    if entry.metadata.is_dir {
        TuiColor::Cyan
    } else {
        match entry.name.split('.').next_back().unwrap_or("") {
            // Systems / compiled langs
            "rs" => TuiColor::Yellow,
            "c" | "h" | "cpp" | "cxx" | "hpp" => TuiColor::Blue,
            "go" => TuiColor::LightBlue,
            "java" | "kt" | "kts" => TuiColor::Red,
            "swift" => TuiColor::LightRed,
            "zig" => TuiColor::LightYellow,

            // Web
            "js" | "mjs" | "cjs" => TuiColor::LightGreen,
            "ts" | "tsx" | "jsx" => TuiColor::LightCyan,
            "html" | "htm" => TuiColor::Magenta,
            "css" | "scss" | "less" => TuiColor::LightMagenta,

            // Scripting / automation
            "py" => TuiColor::LightYellow,
            "sh" | "bash" | "zsh" => TuiColor::Gray,
            "rb" => TuiColor::LightRed,
            "pl" => TuiColor::LightCyan,
            "php" => TuiColor::LightMagenta,
            "lua" => TuiColor::LightBlue,

            // Data / config / markup
            "json" | "toml" | "yaml" | "yml" => TuiColor::Magenta,
            "xml" => TuiColor::LightGreen,
            "csv" => TuiColor::Green,
            "ini" => TuiColor::Gray,

            // Docs
            "md" | "txt" | "rst" | "adoc" => TuiColor::Green,
            "pdf" => TuiColor::Red,

            // Default
            _ => TuiColor::White,
        }
    }
}

/// Format token map entries for TUI display with adaptive layout.
///
/// Returns structured data components used by BOTH the TUI (via widgets)
/// and the CLI (via display_token_map).
pub fn format_token_map_for_tui(
    entries: &[TokenMapEntry],
    total_tokens: usize,
    terminal_width: usize,
    token_format: &TokenFormat,
) -> Vec<TuiTokenMapLine> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Adaptive layout logic
    let terminal_width = terminal_width.max(80);

    // Calculate max token width
    let max_token_width = entries
        .iter()
        .map(|e| format_number(e.tokens, token_format).len())
        .max()
        .unwrap_or(3)
        .max(format_number(total_tokens, token_format).len())
        .max(4);

    // Calculate max name length
    let max_name_length = entries
        .iter()
        .map(|e| {
            let prefix_width = if e.depth == 0 { 3 } else { (e.depth * 2) + 3 };
            prefix_width + UnicodeWidthStr::width(e.name.as_str())
        })
        .max()
        .unwrap_or(20)
        .min(terminal_width / 2);

    // Calculate bar width
    let bar_width = terminal_width
        .saturating_sub(max_token_width + 3 + max_name_length + 2 + 2 + 7)
        .max(15);

    // Initialize parent bars array for hierarchical view
    let mut parent_bars: Vec<String> = vec![String::new(); 20]; // Support up to depth 20
    parent_bars[0] = "█".repeat(bar_width);

    let mut lines = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        // Build tree prefix
        let prefix = build_tree_prefix(entry, entries, i);

        // Format tokens
        let tokens_str = format_number(entry.tokens, token_format);

        // Generate hierarchical bar
        let depth_index = entry.depth;
        let parent_bar = if depth_index > 0 && depth_index - 1 < parent_bars.len() {
            &parent_bars[depth_index - 1]
        } else {
            &parent_bars[0]
        };

        let bar = generate_hierarchical_bar(bar_width, parent_bar, entry.percentage, entry.depth);

        // Update parent bars for children to inherit
        if depth_index < parent_bars.len() {
            parent_bars[depth_index] = bar.clone();
        }

        // Format percentage
        let percentage_str = format!("{:>4.0}%", entry.percentage);

        // Calculate padding for name alignment
        let prefix_display_width = prefix.chars().count();
        let name_padding = max_name_length
            .saturating_sub(prefix_display_width + UnicodeWidthStr::width(entry.name.as_str()));

        // Create name with padding
        // FIX: Ensure name is not empty. If root has empty name, use "." or project name.
        let display_name = if entry.name.is_empty() {
            "."
        } else {
            &entry.name
        };
        let name_with_padding = format!("{}{}", display_name, " ".repeat(name_padding));

        // Determine color
        let name_color = determine_tui_color(entry);

        lines.push(TuiTokenMapLine {
            tokens_part: format!("{:>width$}", tokens_str, width = max_token_width),
            prefix_part: prefix,
            name_part: name_with_padding,
            name_color,
            bar_part: bar,
            percentage_part: percentage_str,
        });
    }

    lines
}

// Generate bar with dust-style depth shading
fn generate_hierarchical_bar(
    bar_width: usize,
    parent_bar: &str,
    percentage: f64,
    depth: usize,
) -> String {
    let filled_chars = ((percentage / 100.0) * bar_width as f64).round() as usize;
    let mut result = String::new();

    // Depth determines which shade to use for parent's solid blocks
    let shade_char = match depth {
        0 => '█', // Root level is solid
        1 => '░', // Level 1: light shade
        2 => '▒', // Level 2: medium shade
        _ => '▓', // Level 3+: dark shade
    };

    let parent_chars: Vec<char> = parent_bar.chars().collect();
    for i in 0..bar_width {
        if i < filled_chars {
            // This entry's contribution is always solid
            result.push('█');
        } else if i < parent_chars.len() {
            // Inherit parent bar but shade it to show nesting
            let parent_char = parent_chars[i];
            if parent_char == '█' {
                result.push(shade_char);
            } else {
                result.push(parent_char); // Keep existing shade
            }
        } else {
            result.push(' ');
        }
    }

    result
}
