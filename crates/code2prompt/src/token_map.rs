use colored::*;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

// Unicode block characters for shading - from full to empty (dust-style)
static BLOCKS: [char; 5] = ['█', '▓', '▒', '░', ' '];

#[derive(Debug, Clone)]
struct TreeNode {
    tokens: usize,
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
    path: String,
}

impl TreeNode {
    fn with_path(path: String) -> Self {
        TreeNode {
            tokens: 0,
            children: BTreeMap::new(),
            is_file: false,
            path,
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
        self.tokens.cmp(&other.tokens)
            .then_with(|| other.depth.cmp(&self.depth))
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for NodePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn generate_token_map_with_limit(
    files: &[serde_json::Value], 
    total_tokens: usize, 
    max_lines: Option<usize>, 
    min_percent: Option<f64>
) -> Vec<TokenMapEntry> {
    // Default values
    let max_lines = max_lines.unwrap_or(20);
    let min_percent = min_percent.unwrap_or(0.1);
    // Build tree structure
    let mut root = TreeNode::with_path(String::new());
    root.tokens = total_tokens;
    
    // Insert all files into the tree
    for file in files {
        if let (Some(path_str), Some(tokens)) = (
            file.get("path").and_then(|p| p.as_str()),
            file.get("token_count").and_then(|t| t.as_u64()),
        ) {
            let tokens = tokens as usize;
            let path = Path::new(path_str);
            
            // Skip the root component if it exists
            let components: Vec<_> = path.components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();
            
            insert_path(&mut root, &components, tokens, String::new());
        }
    }
    
    // Use priority queue to select most significant entries
    let allowed_nodes = select_nodes_to_display(&root, total_tokens, max_lines, min_percent);
    
    // Convert tree to sorted entries for display
    let mut entries = Vec::new();
    rebuild_filtered_tree(&root, String::new(), &allowed_nodes, &mut entries, 0, total_tokens, true);
    
    // Add summary for hidden files if needed
    let displayed_tokens: usize = entries.iter().map(|e| {
        if e.is_file {
            e.tokens
        } else {
            // For directories, only count their direct file children to avoid double counting
            0
        }
    }).sum();
    
    let hidden_tokens = calculate_file_tokens(&root) - displayed_tokens;
    if hidden_tokens > 0 {
        entries.push(TokenMapEntry {
            path: "(other files)".to_string(),
            name: "(other files)".to_string(),
            tokens: hidden_tokens,
            percentage: (hidden_tokens as f64 / total_tokens as f64) * 100.0,
            is_file: false,
            depth: 0,
            is_last: true,
            parent_percentage: 0.0,
        });
    }
    
    entries
}

fn calculate_file_tokens(node: &TreeNode) -> usize {
    if node.is_file {
        node.tokens
    } else {
        node.children.values().map(calculate_file_tokens).sum()
    }
}

fn insert_path(node: &mut TreeNode, components: &[&str], tokens: usize, parent_path: String) {
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
        let child = node.children.entry(file_name).or_insert_with(|| TreeNode::with_path(file_path));
        child.tokens = tokens;
        child.is_file = true;
    } else {
        // This is a directory
        let dir_name = components[0].to_string();
        let dir_path = if parent_path.is_empty() {
            dir_name.clone()
        } else {
            format!("{}/{}", parent_path, dir_name)
        };
        let child = node.children.entry(dir_name).or_insert_with(|| TreeNode::with_path(dir_path.clone()));
        child.tokens += tokens;
        insert_path(child, &components[1..], tokens, dir_path);
    }
}

#[derive(Debug)]
pub struct TokenMapEntry {
    pub path: String,
    pub name: String,
    pub tokens: usize,
    pub percentage: f64,
    pub is_file: bool,
    pub depth: usize,
    pub is_last: bool,
    pub parent_percentage: f64,  // Percentage that parent directories contribute
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
        let name = path.split('/').last().unwrap_or(&path).to_string();
        
        entries.push(TokenMapEntry {
            path: path.clone(),
            name,
            tokens: node.tokens,
            percentage,
            is_file: node.is_file,
            depth,
            is_last,
            parent_percentage: 0.0,
        });
    }
    
    // Process children that are in allowed_nodes
    let mut filtered_children: Vec<_> = node.children.iter()
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
        rebuild_filtered_tree(child, child_path, allowed_nodes, entries, depth + 1, total_tokens, is_last_child);
    }
}

pub fn display_token_map(entries: &[TokenMapEntry], total_tokens: usize) {
    if entries.is_empty() {
        return;
    }

    // Terminal width detection
    let terminal_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(80);

    // Calculate max token width for alignment
    let max_token_width = entries.iter()
        .map(|e| format_tokens(e.tokens).len())
        .max()
        .unwrap_or(3)
        .max(format_tokens(total_tokens).len())
        .max(4);

    // Calculate max name length including tree prefix
    let max_name_length = entries.iter()
        .map(|e| {
            let prefix_width = if e.depth == 0 { 3 } else { (e.depth * 2) + 3 };
            prefix_width + UnicodeWidthStr::width(e.name.as_str())
        })
        .max()
        .unwrap_or(20)
        .min(terminal_width / 2);

    // Calculate bar width with more reasonable spacing
    let bar_width = terminal_width
        .saturating_sub(max_token_width + 3 + max_name_length + 2 + 2 + 5)
        .max(20);

    // Initialize parent bars array
    let mut parent_bars: Vec<String> = vec![String::new(); 10];
    parent_bars[0] = "█".repeat(bar_width);
    
    // Track which depths still have nodes coming (for vertical line continuation)
    let mut has_more_at_depth = vec![false; 10];
    
    // Pre-calculate which depths have more nodes
    for i in 0..entries.len() {
        let entry = &entries[i];
        // Check if there are more siblings at this depth
        for j in (i + 1)..entries.len() {
            if entries[j].depth < entry.depth {
                break; // We've gone up the tree, no more siblings
            }
            if entries[j].depth == entry.depth {
                has_more_at_depth[entry.depth] = true;
                break;
            }
        }
    }

    for (i, entry) in entries.iter().enumerate() {
        // Build tree prefix
        let mut prefix = String::new();
        
        // For each level up to our depth, add the appropriate characters
        for d in 0..entry.depth {
            if d < entry.depth - 1 {
                // Check if we need a vertical line at this depth
                // We need one if there are more nodes at depth d+1 after this node
                let mut needs_line = false;
                for j in (i + 1)..entries.len() {
                    if entries[j].depth <= d {
                        break; // Gone up past this level
                    }
                    if entries[j].depth == d + 1 {
                        needs_line = true;
                        break;
                    }
                }
                if needs_line {
                    prefix.push_str("│ ");
                } else {
                    prefix.push_str("  ");
                }
            } else {
                // This is our immediate parent level
                if entry.is_last {
                    prefix.push_str("└─");
                } else {
                    prefix.push_str("├─");
                }
            }
        }

        // Special handling for root
        if entry.depth == 0 && i == 0 && entry.name != "(other files)" {
            prefix = "┌─".to_string();
        }

        // Check if has children
        let has_children = entries.get(i + 1)
            .map(|next| next.depth > entry.depth)
            .unwrap_or(false);

        // Add the connecting character
        if entry.depth > 0 || entry.name == "(other files)" {
            if has_children {
                prefix.push('┬');
            } else {
                prefix.push('─');
            }
        } else if i == 0 {
            prefix.push('┴');
        }

        prefix.push(' ');

        // Format tokens
        let tokens_str = format_tokens(entry.tokens);

        // Generate hierarchical bar using parent's bar
        let parent_bar = if entry.depth > 0 {
            &parent_bars[entry.depth - 1]
        } else {
            &parent_bars[0]
        };
        
        let bar = generate_hierarchical_bar(
            entry, 
            bar_width, 
            parent_bar,
            entry.percentage,
            entry.depth
        );

        // Update parent bars for this depth level
        if entry.depth < parent_bars.len() {
            parent_bars[entry.depth] = bar.clone();
        }

        // Format percentage
        let percentage_str = format!("{:>4.0}%", entry.percentage);

        // Calculate padding
        let name_display = &entry.name;
        let prefix_display_width = prefix.chars().count();
        let name_padding = max_name_length.saturating_sub(
            prefix_display_width + UnicodeWidthStr::width(name_display.as_str())
        );

        // Print the line - no special case for "(other files)"
        println!(
            "{:>width$}   {}{}{} │{}│ {}",
            tokens_str,
            prefix,
            name_display,
            " ".repeat(name_padding),
            bar,
            percentage_str,
            width = max_token_width
        );
    }
}

// Format token counts with K/M suffixes (dust-style)
fn format_tokens(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{:.0}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.0}K", tokens as f64 / 1_000.0)
    } else {
        format!("{}", tokens)
    }
}

// Generate bar with dust-style depth shading
fn generate_hierarchical_bar(
    entry: &TokenMapEntry, 
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
        1 => ' ',  // Level 1: parent blocks become spaces
        2 => '░',  // Level 2: light shade
        3 => '▒',  // Level 3: medium shade
        _ => '▓',  // Level 4+: dark shade
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1_000), "1K");
        assert_eq!(format_tokens(1_500), "2K");
        assert_eq!(format_tokens(1_000_000), "1M");
        assert_eq!(format_tokens(2_500_000), "3M");
    }
}