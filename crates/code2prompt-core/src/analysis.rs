//! Analysis Engine for Codebase Statistics
//!
//! This module provides a "GraphQL-like" query interface for analyzing codebase data.
//! It offers various aggregation and filtering methods that operate on the raw file list,
//! ensuring complete and accurate statistics regardless of display filtering.

use crate::path::FileEntry;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::path::Path;

// ============================================================================
// Public Data Models
// ============================================================================

/// Represents a single entry in the hierarchical token map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMapEntry {
    pub path: String,
    pub name: String,
    pub tokens: usize,
    pub percentage: f64,
    /// Depth in the tree hierarchy (valid for tree projections)
    pub depth: usize,
    /// Whether this is the last child in its parent's list
    pub is_last: bool,
    pub metadata: EntryMetadata,
}

/// Metadata about a file or directory entry
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub is_dir: bool,
}

/// Statistics for a file extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub file_count: usize,
    pub tokens: usize,
    pub percentage: f64,
}

/// Options for token map generation
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

/// Facade for querying codebase analysis data
///
/// This struct provides a clean, query-like interface for analyzing codebase statistics.
/// Each method operates on the complete file list, ensuring accurate results.
///
/// # Example
///
/// ```rust,ignore
/// let analysis = CodebaseAnalysis::new(files, total_tokens);
/// let token_map = analysis.token_map(TokenMapOptions::default());
/// let extensions = analysis.by_extension();
/// ```
pub struct CodebaseAnalysis<'a> {
    files: &'a [FileEntry],
    total_tokens: usize,
}

impl<'a> CodebaseAnalysis<'a> {
    /// Create a new analysis instance
    ///
    /// # Arguments
    ///
    /// * `files` - Complete list of files from the codebase
    /// * `total_tokens` - Total token count (including structural overhead)
    pub fn new(files: &'a [FileEntry], total_tokens: usize) -> Self {
        Self {
            files,
            total_tokens,
        }
    }

    /// Query: Get hierarchical token map (filtered and sorted)
    ///
    /// Returns a tree-like structure showing token distribution across
    /// directories and files, limited to the most significant entries.
    ///
    /// # Arguments
    ///
    /// * `options` - Filtering options (max lines, min percentage)
    pub fn token_map(&self, options: TokenMapOptions) -> Vec<TokenMapEntry> {
        generate_token_map_internal(self.files, self.total_tokens, options)
    }

    /// Query: Get token statistics by file extension
    ///
    /// Returns aggregated statistics for each file extension found in the codebase.
    /// Unlike the token map, this includes ALL files regardless of filtering.
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
            entry.1 += file.token_count; // token count
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

        // Sort by tokens descending
        result.sort_by(|a, b| b.tokens.cmp(&a.tokens));
        result
    }

    /// Query: Get raw file list
    ///
    /// Provides direct access to the underlying file list for custom processing.
    pub fn raw_files(&self) -> &[FileEntry] {
        self.files
    }
}

// ============================================================================
// Internal Implementation (Token Map Generation)
// ============================================================================

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

#[derive(Debug, Clone, Eq, PartialEq)]
struct NodePriority {
    tokens: usize,
    path: String,
    depth: usize,
}

impl Ord for NodePriority {
    fn cmp(&self, other: &Self) -> Ordering {
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

/// Internal function to generate the token map
fn generate_token_map_internal(
    files: &[FileEntry],
    total_tokens: usize,
    options: TokenMapOptions,
) -> Vec<TokenMapEntry> {
    let mut root = TreeNode::with_path(String::new());
    root.tokens = total_tokens;

    // Build the tree from flat file list
    for file in files {
        let path_str = &file.path;
        let tokens = file.token_count;
        let metadata = EntryMetadata {
            is_dir: file.metadata.is_dir,
        };
        let path = Path::new(path_str);
        let components: Vec<_> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        insert_path(&mut root, &components, tokens, String::new(), metadata);
    }

    // Select nodes to display using priority queue (Dust algorithm)
    let allowed_nodes = select_nodes_to_display(&root, total_tokens, &options);

    // Rebuild as flat list with tree metadata
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

    // Add "other files" aggregation if needed
    let displayed_tokens: usize = entries
        .iter()
        .filter(|e| e.depth == 0)
        .map(|e| e.tokens)
        .sum();

    let hidden_tokens = calculate_file_tokens(&root).saturating_sub(displayed_tokens);

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

    let name = components[0].to_string();
    let current_path = if parent_path.is_empty() {
        name.clone()
    } else {
        format!("{}/{}", parent_path, name)
    };

    let child = node
        .children
        .entry(name)
        .or_insert_with(|| TreeNode::with_path(current_path.clone()));

    if components.len() == 1 {
        // File
        child.tokens = tokens;
        child.metadata = Some(file_metadata);
    } else {
        // Directory
        child.tokens += tokens;
        child.metadata = Some(EntryMetadata { is_dir: true });
        insert_path(child, &components[1..], tokens, current_path, file_metadata);
    }
}

fn select_nodes_to_display(
    root: &TreeNode,
    total_tokens: usize,
    options: &TokenMapOptions,
) -> HashMap<String, usize> {
    let mut heap = BinaryHeap::new();
    let mut allowed_nodes = HashMap::new();
    let min_tokens = (total_tokens as f64 * options.min_percent / 100.0) as usize;

    for child in root.children.values() {
        if child.tokens >= min_tokens {
            heap.push(NodePriority {
                tokens: child.tokens,
                path: child.path.clone(),
                depth: 0,
            });
        }
    }

    while allowed_nodes.len() < options.max_lines.saturating_sub(1) && !heap.is_empty() {
        if let Some(prio) = heap.pop() {
            allowed_nodes.insert(prio.path.clone(), prio.depth);

            if let Some(node) = find_node_by_path(root, &prio.path) {
                for child in node.children.values() {
                    if child.tokens >= min_tokens && !allowed_nodes.contains_key(&child.path) {
                        heap.push(NodePriority {
                            tokens: child.tokens,
                            path: child.path.clone(),
                            depth: prio.depth + 1,
                        });
                    }
                }
            }
        }
    }
    allowed_nodes
}

fn find_node_by_path<'a>(root: &'a TreeNode, path: &str) -> Option<&'a TreeNode> {
    if path.is_empty() {
        return Some(root);
    }
    let mut current = root;
    for component in path.split('/') {
        current = current.children.get(component)?;
    }
    Some(current)
}

fn rebuild_filtered_tree(
    node: &TreeNode,
    path: String,
    allowed_nodes: &HashMap<String, usize>,
    entries: &mut Vec<TokenMapEntry>,
    depth: usize,
    total_tokens: usize,
    is_last: bool,
) {
    if !path.is_empty() && allowed_nodes.contains_key(&path) {
        let name = path.split('/').next_back().unwrap_or(&path).to_string();
        entries.push(TokenMapEntry {
            path: path.clone(),
            name,
            tokens: node.tokens,
            percentage: (node.tokens as f64 / total_tokens as f64) * 100.0,
            depth,
            is_last,
            metadata: node.metadata.unwrap_or(EntryMetadata { is_dir: true }),
        });
    }

    let mut filtered_children: Vec<_> = node
        .children
        .iter()
        .filter(|(_, child)| allowed_nodes.contains_key(&child.path))
        .collect();

    filtered_children.sort_by(|a, b| b.1.tokens.cmp(&a.1.tokens));

    let count = filtered_children.len();
    for (i, (name, child)) in filtered_children.into_iter().enumerate() {
        let child_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", path, name)
        };
        rebuild_filtered_tree(
            child,
            child_path,
            allowed_nodes,
            entries,
            depth + 1,
            total_tokens,
            i == count - 1,
        );
    }
}
