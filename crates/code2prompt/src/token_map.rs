//! Token map visualization and analysis.
//!
//! This module provides functionality for generating and displaying visual token maps
//! that show how tokens are distributed across files in a codebase. It creates
//! hierarchical tree structures with visual bars and colors, similar to disk usage
//! analyzers but for token consumption.
use code2prompt_core::path::FileEntry;
#[cfg(windows)]
use log::error;
use lscolors::{Indicator, LsColors};
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
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

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EntryMetadata {
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
struct TreeNode {
    tokens: usize,
    children: BTreeMap<String, TreeNode>,
    path: String,
    metadata: Option<EntryMetadata>,
}

impl TreeNode {
    fn with_path(path: String) -> Self {
        TreeNode {
            tokens: 0,
            children: BTreeMap::new(),
            path,
            metadata: None,
        }
    }
}

// For priority queue ordering
#[derive(Debug, Clone, Eq, PartialEq)]
struct NodePriority {
    tokens: usize,
    path: String,
    depth: usize,
}

impl Ord for NodePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by tokens (descending), then by depth (ascending), then by path
        self.tokens
            .cmp(&other.tokens)
            .then_with(|| other.depth.cmp(&self.depth))
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for NodePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Generate a hierarchical token map with optional display limits.
///
/// Creates a tree structure showing token distribution across files and directories,
/// with optional limits on the number of entries and minimum percentage thresholds
/// for inclusion in the output.
///
/// # Arguments
///
/// * `files` - Array of file metadata from the code2prompt session
/// * `total_tokens` - Total token count for percentage calculations
/// * `max_lines` - Maximum number of entries to return (None for unlimited)
/// * `min_percent` - Minimum percentage threshold for inclusion (None for no limit)
///
/// # Returns
///
/// * `Vec<TokenMapEntry>` - Hierarchical list of token map entries ready for display
pub fn generate_token_map_with_limit(
    files: &[FileEntry],
    total_tokens: usize,
    max_lines: Option<usize>,
    min_percent: Option<f64>,
) -> Vec<TokenMapEntry> {
    let max_lines = max_lines.unwrap_or(20);
    let min_percent = min_percent.unwrap_or(0.1);

    let mut root = TreeNode::with_path(String::new());
    root.tokens = total_tokens;

    // Insert all files into the tree
    for file in files {
        let path_str = &file.path;
        let tokens = file.token_count;
        let metadata = EntryMetadata {
            is_dir: file.metadata.is_dir,
        };

        let path = Path::new(path_str);

        // Skip the root component if it exists
        let components: Vec<_> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        insert_path(&mut root, &components, tokens, String::new(), metadata);
    }

    // Use priority queue to select most significant entries
    let allowed_nodes = select_nodes_to_display(&root, total_tokens, max_lines, min_percent);

    // Convert tree to sorted entries for display
    let mut entries = Vec::new();
    rebuild_filtered_tree(
        &root,
        String::new(),
        &allowed_nodes,
        &mut entries,
        0,
        total_tokens,
        true,
    );

    // Add summary for hidden files if needed
    let displayed_tokens: usize = entries
        .iter()
        .map(|e| {
            if !e.metadata.is_dir {
                e.tokens
            } else {
                // For directories, only count their direct file children to avoid double counting
                0
            }
        })
        .sum();

    let hidden_tokens = calculate_file_tokens(&root) - displayed_tokens;
    if hidden_tokens > 0 {
        entries.push(TokenMapEntry {
            path: "(other files)".to_string(),
            name: "(other files)".to_string(),
            tokens: hidden_tokens,
            percentage: (hidden_tokens as f64 / total_tokens as f64) * 100.0,
            depth: 0,
            is_last: true,
            metadata: EntryMetadata { is_dir: false },
        });
    }

    entries
}

fn calculate_file_tokens(node: &TreeNode) -> usize {
    if node.metadata.is_some_and(|m| !m.is_dir) {
        node.tokens
    } else {
        node.children.values().map(calculate_file_tokens).sum()
    }
}

fn insert_path(
    node: &mut TreeNode,
    components: &[&str],
    tokens: usize,
    parent_path: String,
    file_metadata: EntryMetadata,
) {
    if components.is_empty() {
        return;
    }

    if components.len() == 1 {
        // This is a file
        let file_name = components[0].to_string();
        let file_path = if parent_path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", parent_path, file_name)
        };
        let child = node
            .children
            .entry(file_name)
            .or_insert_with(|| TreeNode::with_path(file_path));
        child.tokens = tokens;
        child.metadata = Some(file_metadata);
    } else {
        // This is a directory
        let dir_name = components[0].to_string();
        let dir_path = if parent_path.is_empty() {
            dir_name.clone()
        } else {
            format!("{}/{}", parent_path, dir_name)
        };
        let child = node
            .children
            .entry(dir_name)
            .or_insert_with(|| TreeNode::with_path(dir_path.clone()));
        child.tokens += tokens;
        child.metadata = Some(EntryMetadata { is_dir: true });
        insert_path(child, &components[1..], tokens, dir_path, file_metadata);
    }
}

#[derive(Debug, Clone)]
pub struct TokenMapEntry {
    pub path: String,
    pub name: String,
    pub tokens: usize,
    pub percentage: f64,
    pub depth: usize,
    pub is_last: bool,
    pub metadata: EntryMetadata,
}

/// Select nodes to display using priority queue
fn select_nodes_to_display(
    root: &TreeNode,
    total_tokens: usize,
    max_lines: usize,
    min_percent: f64,
) -> HashMap<String, usize> {
    let mut heap = BinaryHeap::new();
    let mut allowed_nodes = HashMap::new();
    let min_tokens = (total_tokens as f64 * min_percent / 100.0) as usize;

    // Start with root children
    for child in root.children.values() {
        if child.tokens >= min_tokens {
            heap.push(NodePriority {
                tokens: child.tokens,
                path: child.path.clone(),
                depth: 0,
            });
        }
    }

    // Process nodes by priority
    while allowed_nodes.len() < max_lines.saturating_sub(1) && !heap.is_empty() {
        if let Some(node_priority) = heap.pop() {
            allowed_nodes.insert(node_priority.path.clone(), node_priority.depth);

            // Find the node in the tree and add its children
            if let Some(node) = find_node_by_path(root, &node_priority.path) {
                for child in node.children.values() {
                    if child.tokens >= min_tokens && !allowed_nodes.contains_key(&child.path) {
                        heap.push(NodePriority {
                            tokens: child.tokens,
                            path: child.path.clone(),
                            depth: node_priority.depth + 1,
                        });
                    }
                }
            }
        }
    }

    allowed_nodes
}

/// Find a node by its path
fn find_node_by_path<'a>(root: &'a TreeNode, path: &str) -> Option<&'a TreeNode> {
    if path.is_empty() {
        return Some(root);
    }

    let components: Vec<&str> = path.split('/').collect();
    let mut current = root;

    for component in components {
        match current.children.get(component) {
            Some(child) => current = child,
            None => return None,
        }
    }

    Some(current)
}

/// Rebuild tree with only allowed nodes
fn rebuild_filtered_tree(
    node: &TreeNode,
    path: String,
    allowed_nodes: &HashMap<String, usize>,
    entries: &mut Vec<TokenMapEntry>,
    depth: usize,
    total_tokens: usize,
    is_last: bool,
) {
    // Check if this node should be included
    if !path.is_empty() && allowed_nodes.contains_key(&path) {
        let percentage = (node.tokens as f64 / total_tokens as f64) * 100.0;
        let name = path.split('/').next_back().unwrap_or(&path).to_string();

        let metadata = node.metadata.unwrap_or(EntryMetadata { is_dir: true });

        entries.push(TokenMapEntry {
            path: path.clone(),
            name,
            tokens: node.tokens,
            percentage,
            depth,
            is_last,
            metadata,
        });
    }

    // Process children that are in allowed_nodes
    let mut filtered_children: Vec<_> = node
        .children
        .iter()
        .filter(|(_, child)| allowed_nodes.contains_key(&child.path))
        .collect();

    // Sort by tokens descending
    filtered_children.sort_by(|a, b| b.1.tokens.cmp(&a.1.tokens));

    let child_count = filtered_children.len();
    for (i, (name, child)) in filtered_children.into_iter().enumerate() {
        let child_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", path, name)
        };

        let is_last_child = i == child_count - 1;
        rebuild_filtered_tree(
            child,
            child_path,
            allowed_nodes,
            entries,
            depth + 1,
            total_tokens,
            is_last_child,
        );
    }
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
                    .style_for_path(std::path::Path::new(&entry.path))
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
                .take_while(|entry| entry.depth > d)
                .any(|entry| entry.depth == d + 1);
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
    // TUI needs a bit more space than CLI to prevent the percentage column from overflowing
    let bar_width = terminal_width
        .saturating_sub(max_token_width + 3 + max_name_length + 2 + 2 + 7) // +2 more chars for TUI
        .max(15); // Minimum bar width reduced slightly for TUI

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
