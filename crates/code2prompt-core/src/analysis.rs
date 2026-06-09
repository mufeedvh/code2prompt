//! Analysis Engine for Codebase Statistics
//!
//! This module computes TOPOLOGY and STRUCTURE.
//! It knows nothing about ASCII art, colors, or how to draw a tree.
//!
//! Inspired by dust and JackYoustra's implementation

use crate::path::FileEntry;
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering,Reverse};
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::path::Path;

// ============================================================================
// Public Data Models
// ============================================================================

/// Represents a topological entry in the statistics tree.
///
/// CLEAN ARCHITECTURE: This struct contains NO visual artifacts (like "│ │").
/// It only describes the node's properties and its position in the hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMapEntry {
    pub path: String,
    pub name: String,
    pub tokens: usize,
    pub percentage: f64,

    // Topological Metadata
    pub depth: usize,
    /// Is this node the last child of its parent? (Crucial for drawing logic)
    pub is_last_child: bool,
    /// Does this node have visible children in the filtered set?
    pub has_children: bool,

    pub metadata: EntryMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub file_count: usize,
    pub tokens: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct TokenMapOptions {
    pub max_lines: usize,
    pub min_percent: f64,
}

impl Default for TokenMapOptions {
    fn default() -> Self {
        Self {
            max_lines: 20,
            min_percent: 0.1,
        }
    }
}

// ============================================================================
// CodebaseAnalysis Facade
// ============================================================================

pub struct CodebaseAnalysis<'a> {
    files: &'a [FileEntry],
    total_tokens: usize,
}

impl<'a> CodebaseAnalysis<'a> {
    pub fn new(files: &'a [FileEntry], total_tokens: usize) -> Self {
        Self {
            files,
            total_tokens,
        }
    }

    /// Generates a flat list of entries representing the filtered tree.
    /// Uses the dust-inspired algorithm to show the most significant entries.
    pub fn token_map(&self, options: TokenMapOptions) -> Vec<TokenMapEntry> {
        // 1. Find common path prefix to strip (for absolute paths)
        let common_prefix = find_common_path_prefix(self.files);
        
        // 2. Build the tree structure
        let mut root = TreeNode::new(String::new());
        root.tokens = self.total_tokens;

        // Insert all files into the tree
        for file in self.files {
            let path = Path::new(&file.path);
            
            // Strip common prefix if present
            let path_to_use = if let Some(prefix) = &common_prefix {
                path.strip_prefix(prefix).unwrap_or(path)
            } else {
                path
            };
            
            let components: Vec<&str> = path_to_use
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .filter(|c| *c != "/" && !c.is_empty()) // Skip root slash and empty components
                .collect();

            if !components.is_empty() {
                insert_path(
                    &mut root,
                    &components,
                    file.token_count,
                    String::new(),
                    EntryMetadata {
                        is_dir: file.metadata.is_dir,
                    },
                );
            }
        }

        // 2. Select nodes to display using priority queue (dust algorithm)
        let allowed_nodes =
            select_nodes_to_display(&root, self.total_tokens, &options);

        // 3. Flatten the tree to entries, respecting the selection
        let mut entries = Vec::new();
        rebuild_filtered_tree(
            &root,
            String::new(),
            &allowed_nodes,
            &mut entries,
            0,
            self.total_tokens,
            true,
        );

        // 4. Calculate tokens for files actually displayed (avoid double-counting dirs)
        let displayed_file_tokens: usize = entries
            .iter()
            .filter(|e| !e.metadata.is_dir)
            .map(|e| e.tokens)
            .sum();

        // 5. Calculate total file tokens in the tree (not directory sums)
        let total_file_tokens = calculate_file_tokens(&root);

        // 6. Add "Other files" aggregation if we filtered things out
        let hidden_tokens = total_file_tokens.saturating_sub(displayed_file_tokens);
        if hidden_tokens > 0 {
            // Mark the previous last item as not last anymore
            if let Some(last) = entries.last_mut()
                && last.depth == 0 {
                    last.is_last_child = false;
                }

            entries.push(TokenMapEntry {
                path: "(other files)".to_string(),
                name: "(other files)".to_string(),
                tokens: hidden_tokens,
                percentage: (hidden_tokens as f64 / self.total_tokens as f64) * 100.0,
                depth: 0,
                is_last_child: true,
                has_children: false,
                metadata: EntryMetadata { is_dir: false },
            });
        }

        entries
    }

    /// Aggregate tokens by file extension
    pub fn by_extension(&self) -> Vec<ExtensionStat> {
        let mut stats: HashMap<String, (usize, usize)> = HashMap::new();

        for file in self.files {
            if file.metadata.is_dir {
                continue;
            }

            let ext = Path::new(&file.path)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("(no extension)")
                .to_string();

            let entry = stats.entry(ext).or_insert((0, 0));
            entry.0 += 1; // file count
            entry.1 += file.token_count; // tokens
        }

        let mut result: Vec<ExtensionStat> = stats
            .into_iter()
            .map(|(ext, (count, tokens))| ExtensionStat {
                extension: ext,
                file_count: count,
                tokens,
                percentage: (tokens as f64 / self.total_tokens as f64) * 100.0,
            })
            .collect();

        result.sort_by_key(|b| Reverse(b.tokens));
        result
    }

    /// Get raw file entries
    pub fn raw_files(&self) -> &[FileEntry] {
        self.files
    }
}

// ============================================================================
// Internal Tree Structure
// ============================================================================

#[derive(Debug, Clone)]
struct TreeNode {
    tokens: usize,
    children: BTreeMap<String, TreeNode>, // BTreeMap for deterministic ordering!
    path: String,
    metadata: Option<EntryMetadata>,
}

impl TreeNode {
    fn new(path: String) -> Self {
        TreeNode {
            tokens: 0,
            children: BTreeMap::new(),
            path,
            metadata: None,
        }
    }
}

/// Insert a file path into the tree, accumulating tokens up the hierarchy
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
        // This is a file (leaf node)
        let file_name = components[0].to_string();
        let file_path = if parent_path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", parent_path, file_name)
        };

        let child = node
            .children
            .entry(file_name)
            .or_insert_with(|| TreeNode::new(file_path));
        child.tokens = tokens;
        child.metadata = Some(file_metadata);
    } else {
        // This is a directory (intermediate node)
        let dir_name = components[0].to_string();
        let dir_path = if parent_path.is_empty() {
            dir_name.clone()
        } else {
            format!("{}/{}", parent_path, dir_name)
        };

        let child = node
            .children
            .entry(dir_name)
            .or_insert_with(|| TreeNode::new(dir_path.clone()));
        child.tokens += tokens; // Accumulate tokens for directory
        child.metadata = Some(EntryMetadata { is_dir: true });

        // Recurse into the next level
        insert_path(child, &components[1..], tokens, dir_path, file_metadata);
    }
}

// ============================================================================
// Priority Queue Filtering (Dust Algorithm)
// ============================================================================

/// Helper for Priority Queue - sorts nodes by tokens (descending)
#[derive(Debug, Clone, Eq, PartialEq)]
struct NodePriority {
    tokens: usize,
    path: String,
    depth: usize,
}

impl Ord for NodePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by tokens (descending), then by depth (ascending for ties), then by path
        self.tokens
            .cmp(&other.tokens)
            .then_with(|| other.depth.cmp(&self.depth)) // Prefer shallower when equal tokens
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for NodePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Select nodes to display using priority queue (dust-inspired)
fn select_nodes_to_display(
    root: &TreeNode,
    total_tokens: usize,
    options: &TokenMapOptions,
) -> HashMap<String, usize> {
    let mut heap = BinaryHeap::new();
    let mut allowed_nodes = HashMap::new();
    let min_tokens = (total_tokens as f64 * options.min_percent / 100.0) as usize;

    // Start with root's children
    for child in root.children.values() {
        if child.tokens >= min_tokens {
            heap.push(NodePriority {
                tokens: child.tokens,
                path: child.path.clone(),
                depth: 0,
            });
        }
    }

    // Process nodes by priority (highest tokens first)
    while allowed_nodes.len() < options.max_lines.saturating_sub(1) && !heap.is_empty() {
        if let Some(node_priority) = heap.pop() {
            allowed_nodes.insert(node_priority.path.clone(), node_priority.depth);

            // Find this node in the tree and add its children to the heap
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

/// Find a node in the tree by its path
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

/// Rebuild tree with only allowed nodes, creating flat list for display
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

        // Check if this node has children that will be displayed
        let has_visible_children = node
            .children
            .values()
            .any(|child| allowed_nodes.contains_key(&child.path));

        entries.push(TokenMapEntry {
            path: path.clone(),
            name,
            tokens: node.tokens,
            percentage,
            depth: depth.saturating_sub(1), // Adjust because root is depth -1 conceptually
            is_last_child: is_last,
            has_children: has_visible_children,
            metadata,
        });
    }

    // Process children that are in allowed_nodes
    let mut filtered_children: Vec<(&String, &TreeNode)> = node
        .children
        .iter()
        .filter(|(_, child)| allowed_nodes.contains_key(&child.path))
        .collect();

    // Sort by tokens descending
    filtered_children.sort_by_key(|b| Reverse(b.1.tokens));

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

/// Calculate total tokens from FILES only (not directory aggregates)
fn calculate_file_tokens(node: &TreeNode) -> usize {
    if node.metadata.is_some_and(|m| !m.is_dir) {
        // This is a file - count its tokens
        node.tokens
    } else {
        // This is a directory - sum its children's file tokens
        node.children.values().map(calculate_file_tokens).sum()
    }
}

/// Find the common path prefix to strip from all file paths
/// This is useful when working with absolute paths to show relative structure
/// Only strips absolute path prefixes, not relative ones
fn find_common_path_prefix(files: &[FileEntry]) -> Option<std::path::PathBuf> {
    if files.is_empty() {
        return None;
    }

    // Get the first file's path components as a starting point
    let first_path = Path::new(&files[0].path);
    
    // Only strip prefix for absolute paths
    if !first_path.is_absolute() {
        return None;
    }
    
    let first_components: Vec<_> = first_path.components().collect();

    // Find the longest common prefix across all file paths
    let mut common_len = first_components.len();
    
    for file in files.iter().skip(1) {
        let path = Path::new(&file.path);
        let components: Vec<_> = path.components().collect();
        
        // Find how many components match with our current common prefix
        let mut matching = 0;
        for (a, b) in first_components.iter().zip(components.iter()) {
            if a == b {
                matching += 1;
            } else {
                break;
            }
        }
        
        common_len = common_len.min(matching);
        
        if common_len == 0 {
            return None; // No common prefix
        }
    }

    // Don't strip everything - leave at least one level
    if common_len >= first_components.len() {
        common_len = first_components.len().saturating_sub(1);
    }

    if common_len == 0 {
        return None;
    }

    // Build the common prefix path
    let mut prefix = std::path::PathBuf::new();
    for component in first_components.iter().take(common_len) {
        prefix.push(component);
    }

    Some(prefix)
}
