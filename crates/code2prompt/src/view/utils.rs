//! Utility functions for view operations.
//!
//! This module contains helper functions for view-related operations
//! that don't fit into the formatters module.

use ratatui::style::Color;

/// Get color for file extension
#[allow(dead_code)] // Will be used when widgets are refactored
pub fn get_extension_color(extension: &str) -> Color {
    match extension {
        ".rs" => Color::LightRed,
        ".md" | ".txt" | ".rst" => Color::Green,
        ".toml" | ".json" | ".yaml" | ".yml" => Color::Magenta,
        ".js" | ".ts" | ".jsx" | ".tsx" => Color::Cyan,
        ".py" => Color::LightYellow,
        ".go" => Color::LightBlue,
        ".java" | ".kt" => Color::Red,
        ".cpp" | ".c" | ".h" => Color::Blue,
        _ => Color::White,
    }
}

/// Calculate scroll position for viewport
#[allow(dead_code)] // Will be used when widgets are refactored
pub fn calculate_scroll_position(
    cursor: usize,
    current_scroll: usize,
    viewport_height: usize,
    total_items: usize,
) -> usize {
    if total_items == 0 {
        return 0;
    }

    // If cursor is above viewport, scroll up
    if cursor < current_scroll {
        cursor
    }
    // If cursor is below viewport, scroll down
    else if cursor >= current_scroll + viewport_height {
        cursor.saturating_sub(viewport_height - 1)
    }
    // Cursor is within viewport, no change needed
    else {
        current_scroll
    }
    .min(total_items.saturating_sub(viewport_height))
}

/// Format scroll indicator for lists
#[allow(dead_code)] // Will be used when widgets are refactored
pub fn format_scroll_indicator(
    title: &str,
    current_start: usize,
    current_end: usize,
    total_items: usize,
    viewport_height: usize,
) -> String {
    if total_items > viewport_height {
        format!(
            "{} | Showing {}-{} of {}",
            title,
            current_start + 1,
            current_end,
            total_items
        )
    } else {
        format!("{} ({})", title, total_items)
    }
}
