//! Token map visualization (View Layer).
//!
//! This module provides rendering functionality for displaying token maps
//! in both CLI and TUI modes. The core logic for generating token maps
//! has been moved to code2prompt-core/analysis.rs.

use code2prompt_core::analysis::TokenMapEntry;
use lscolors::{Indicator, LsColors};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

/// Color information for TUI rendering
#[derive(Debug, Clone)]
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

/// Formatted line for TUI token map display with separate components
#[derive(Debug, Clone)]
pub struct TuiTokenMapLine {
    pub tokens_part: String,
    pub prefix_part: String,
    pub name_part: String,
    pub name_color: TuiColor,
    pub bar_part: String,
    pub percentage_part: String,
}

fn should_enable_colors() -> bool {
    // Check NO_COLOR environment variable (https://no-color.org/)
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }

    // Check if we're in a terminal
    if terminal_size::terminal_size().is_none() {
        return false;
    }

    // On Windows, enable ANSI support
    #[cfg(windows)]
    {
        use log::error;
        match ansi_term::enable_ansi_support() {
            Ok(_) => true,
            Err(_) => {
                error!("This version of Windows does not support ANSI colors");
                false
            }
        }
    }

    #[cfg(not(windows))]
    {
        true
    }
}

/// Display a visual token map with colors and hierarchical tree structure.
///
/// Renders the token map entries as a formatted tree with visual progress bars,
/// colors based on file types, and proper Unicode tree drawing characters.
/// Automatically adapts to terminal width and applies appropriate colors.
///
/// # Arguments
///
/// * `entries` - The token map entries to display
/// * `total_tokens` - Total token count for percentage calculations
pub fn display_token_map(entries: &[TokenMapEntry], total_tokens: usize) {
    if entries.is_empty() {
        return;
    }

    // Initialize LsColors from environment
    let ls_colors = LsColors::from_env().unwrap_or_default();
    let colors_enabled = should_enable_colors();

    // Terminal width detection
    let terminal_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(80);

    // Calculate max token width for alignment
    let max_token_width = entries
        .iter()
        .map(|e| format_tokens(e.tokens).len())
        .max()
        .unwrap_or(3)
        .max(format_tokens(total_tokens).len())
        .max(4);

    // Calculate max name length including tree prefix
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
        .saturating_sub(max_token_width + 3 + max_name_length + 2 + 2 + 5)
        .max(20);

    // Initialize parent bars array
    let mut parent_bars: Vec<String> = vec![String::new(); 10];
    parent_bars[0] = "█".repeat(bar_width);

    for (i, entry) in entries.iter().enumerate() {
        // Build tree prefix using shared logic
        let prefix = build_tree_prefix(entry, entries, i);

        // Format tokens
        let tokens_str = format_tokens(entry.tokens);

        // Generate hierarchical bar
        let parent_bar = if entry.depth > 0 {
            &parent_bars[entry.depth - 1]
        } else {
            &parent_bars[0]
        };

        let bar = generate_hierarchical_bar(bar_width, parent_bar, entry.percentage, entry.depth);

        // Update parent bars
        if entry.depth < parent_bars.len() {
            parent_bars[entry.depth] = bar.clone();
        }

        // Format percentage
        let percentage_str = format!("{:>4.0}%", entry.percentage);

        // Calculate padding for name
        let prefix_display_width = prefix.chars().count();
        let name_padding = max_name_length
            .saturating_sub(prefix_display_width + UnicodeWidthStr::width(entry.name.as_str()));

        // Create name with padding FIRST
        let name_with_padding = format!("{}{}", entry.name, " ".repeat(name_padding));

        // THEN apply colors to the name+padding combination
        let colored_name_with_padding = if colors_enabled && entry.name != "(other files)" {
            // Use our cached metadata to choose the coloring strategy
            let ansi_style = if entry.metadata.is_dir {
                // For directories, we know the type. No need to hit the filesystem.
                ls_colors
                    .style_for_indicator(Indicator::Directory)
                    .map(|s| s.to_ansi_term_style())
                    .unwrap_or_default()
            } else {
                // For files, rely on extension-based styling (no filesystem stat).
                ls_colors
                    .style_for_path(Path::new(&entry.path))
                    .map(lscolors::Style::to_ansi_term_style)
                    .unwrap_or_default()
            };

            // Apply style to name WITH padding
            format!("{}", ansi_style.paint(name_with_padding))
        } else {
            name_with_padding
        };

        eprintln!(
            "{:>width$}   {}{} │{}│ {}",
            tokens_str,
            prefix,
            colored_name_with_padding,
            bar,
            percentage_str,
            width = max_token_width
        );
    }
}

/// Build tree prefix for an entry (shared logic for CLI and TUI)
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

    // Special handling for root
    if entry.depth == 0 && index == 0 && entry.name != "(other files)" {
        prefix = "┌─".to_string();
    }

    // Check if has children
    let has_children = entries
        .get(index + 1)
        .map(|next| next.depth > entry.depth)
        .unwrap_or(false);

    // Add the connecting character
    if entry.depth > 0 || entry.name == "(other files)" {
        if has_children {
            prefix.push('┬');
        } else {
            prefix.push('─');
        }
    } else if index == 0 {
        prefix.push('┴');
    }

    prefix.push(' ');
    prefix
}

/// Determine TUI color for an entry based on file type and extension
fn determine_tui_color(entry: &TokenMapEntry) -> TuiColor {
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
/// Creates formatted lines with tree structure and color information suitable
/// for rendering in a TUI interface using ratatui. This function uses the same
/// adaptive layout logic as the CLI version but returns structured data components
/// instead of printing directly.
///
/// # Arguments
///
/// * `entries` - The token map entries to format
/// * `total_tokens` - Total token count for percentage calculations
/// * `terminal_width` - Width of the terminal/TUI area for adaptive layout
///
/// # Returns
///
/// * `Vec<TuiTokenMapLine>` - Formatted lines ready for TUI rendering
pub fn format_token_map_for_tui(
    entries: &[TokenMapEntry],
    total_tokens: usize,
    terminal_width: usize,
) -> Vec<TuiTokenMapLine> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Use the same adaptive layout logic as CLI
    let terminal_width = terminal_width.max(80); // Minimum width

    // Calculate max token width for alignment (same as CLI)
    let max_token_width = entries
        .iter()
        .map(|e| format_tokens(e.tokens).len())
        .max()
        .unwrap_or(3)
        .max(format_tokens(total_tokens).len())
        .max(4);

    // Calculate max name length including tree prefix (same as CLI)
    let max_name_length = entries
        .iter()
        .map(|e| {
            let prefix_width = if e.depth == 0 { 3 } else { (e.depth * 2) + 3 };
            prefix_width + UnicodeWidthStr::width(e.name.as_str())
        })
        .max()
        .unwrap_or(20)
        .min(terminal_width / 2);

    // Calculate bar width (adjusted for TUI to prevent overflow)
    let bar_width = terminal_width
        .saturating_sub(max_token_width + 3 + max_name_length + 2 + 2 + 7)
        .max(15);

    // Initialize parent bars array (same as CLI)
    let mut parent_bars: Vec<String> = vec![String::new(); 10];
    parent_bars[0] = "█".repeat(bar_width);

    let mut lines = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        // Build tree prefix using shared logic
        let prefix = build_tree_prefix(entry, entries, i);

        // Format tokens
        let tokens_str = format_tokens(entry.tokens);

        // Generate hierarchical bar (same as CLI)
        let parent_bar = if entry.depth > 0 {
            &parent_bars[entry.depth - 1]
        } else {
            &parent_bars[0]
        };

        let bar = generate_hierarchical_bar(bar_width, parent_bar, entry.percentage, entry.depth);

        // Update parent bars (same as CLI)
        if entry.depth < parent_bars.len() {
            parent_bars[entry.depth] = bar.clone();
        }

        // Format percentage
        let percentage_str = format!("{:>4.0}%", entry.percentage);

        // Calculate padding for name (same as CLI)
        let prefix_display_width = prefix.chars().count();
        let name_padding = max_name_length
            .saturating_sub(prefix_display_width + UnicodeWidthStr::width(entry.name.as_str()));

        // Create name with padding
        let name_with_padding = format!("{}{}", entry.name, " ".repeat(name_padding));

        // Determine color based on entry type and extension
        let name_color = determine_tui_color(entry);

        // Create structured components for TUI rendering
        lines.push(TuiTokenMapLine {
            tokens_part: format!("{:>width$}", tokens_str, width = max_token_width),
            prefix_part: prefix,
            name_part: name_with_padding,
            name_color,
            bar_part: format!("│{}│", bar),
            percentage_part: percentage_str,
        });
    }

    lines
}

// Format token counts with K/M suffixes (dust-style)
fn format_tokens(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        let millions = (tokens + 500_000) / 1_000_000;
        format!("{}M", millions)
    } else if tokens >= 1_000 {
        let thousands = (tokens + 500) / 1_000;
        format!("{}K", thousands)
    } else {
        format!("{}", tokens)
    }
}

// Generate bar with dust-style depth shading
fn generate_hierarchical_bar(
    bar_width: usize,
    parent_bar: &str,
    percentage: f64,
    depth: usize,
) -> String {
    // Calculate how many characters should be filled for this entry
    let filled_chars = ((percentage / 100.0) * bar_width as f64).round() as usize;
    let mut result = String::new();

    // Depth determines which shade to use for parent's solid blocks
    let shade_char = match depth.max(1) {
        1 => ' ', // Level 1: parent blocks become spaces
        2 => '░', // Level 2: light shade
        3 => '▒', // Level 3: medium shade
        _ => '▓', // Level 4+: dark shade
    };

    // Process each character position
    let parent_chars: Vec<char> = parent_bar.chars().collect();
    for i in 0..bar_width {
        if i < filled_chars {
            // This is our filled portion - always solid
            result.push('█');
        } else if i < parent_chars.len() {
            // This is parent's portion
            let parent_char = parent_chars[i];
            if parent_char == '█' {
                // Replace parent's solid blocks with our shade
                result.push(shade_char);
            } else {
                // Keep parent's existing shading
                result.push(parent_char);
            }
        } else {
            // Beyond parent's bar - empty
            result.push(' ');
        }
    }

    result
}
