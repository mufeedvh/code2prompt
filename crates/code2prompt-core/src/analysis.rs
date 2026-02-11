//! Analysis Engine for Codebase Statistics
//!
//! This module computes TOPOLOGY and STRUCTURE.
//! It knows nothing about ASCII art, colors, or how to draw a tree.

use crate::path::FileEntry;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
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
    pub fn token_map(&self, options: TokenMapOptions) -> Vec<TokenMapEntry> {
        // 1. Build the full internal tree
        let mut root = Node::new("", true);
        for file in self.files {
            root.insert(&file.path, file.token_count, file.metadata.is_dir);
        }

        // 2. Calculate recursive token sums (Post-Order Traversal)
        root.calculate_sizes();

        // 3. Select which nodes to keep (The "Dust" Algorithm)
        let mut keep_set = HashMap::new();
        // Always keep root (depth 0 is virtual, so we track children of root)
        select_significant_nodes(&root, self.total_tokens, &options, &mut keep_set);

        // 4. Flatten the tree into the public struct, respecting the selection
        let mut entries = Vec::new();
        let mut covered_tokens = 0;

        flatten_tree(
            &root,
            0,
            true, // Root is functionally "last" in its context
            &keep_set,
            self.total_tokens,
            &mut entries,
            &mut covered_tokens,
        );

        // 5. Add "Other files" aggregation if we filtered things out
        let remainder = self.total_tokens.saturating_sub(covered_tokens);
        if remainder > 0 {
            // Adjust the previous last item to not be last anymore
            if let Some(last) = entries.last_mut() {
                // Only if at root level (depth 0)
                if last.depth == 0 {
                    last.is_last_child = false;
                }
            }

            entries.push(TokenMapEntry {
                path: "(other files)".to_string(),
                name: "(other files)".to_string(),
                tokens: remainder,
                percentage: (remainder as f64 / self.total_tokens as f64) * 100.0,
                depth: 0,
                is_last_child: true,
                has_children: false,
                metadata: EntryMetadata { is_dir: false },
            });
        }

        entries
    }

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
            entry.0 += 1;
            entry.1 += file.token_count;
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

        result.sort_by(|a, b| b.tokens.cmp(&a.tokens));
        result
    }

    pub fn raw_files(&self) -> &[FileEntry] {
        self.files
    }
}

// ============================================================================
// Internal Logic
// ============================================================================

struct Node {
    name: String,
    path: String,
    token_sum: usize,
    is_dir: bool,
    children: HashMap<String, Node>,
}

impl Node {
    fn new(name: &str, is_dir: bool) -> Self {
        Self {
            name: name.to_string(),
            path: String::new(), // set during insert
            token_sum: 0,
            is_dir,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, path: &str, tokens: usize, is_dir: bool) {
        let parts: Vec<&str> = path.split('/').collect();
        self.insert_recursive(&parts, path, tokens, is_dir);
    }

    fn insert_recursive(
        &mut self,
        parts: &[&str],
        full_path: &str,
        tokens: usize,
        is_file_dir: bool,
    ) {
        if parts.is_empty() {
            return;
        }

        let name = parts[0];
        let is_last_part = parts.len() == 1;

        let child = self.children.entry(name.to_string()).or_insert_with(|| {
            // Internal nodes are dirs, leaf is whatever the file says
            let child_is_dir = if is_last_part { is_file_dir } else { true };
            Node::new(name, child_is_dir)
        });

        // Reconstruct path for the child
        child.path = if self.path.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.path, name)
        };

        if is_last_part {
            child.token_sum = tokens; // Leaf gets the tokens
        } else {
            child.insert_recursive(&parts[1..], full_path, tokens, is_file_dir);
        }
    }

    /// Sums up tokens from children to parents
    fn calculate_sizes(&mut self) {
        if !self.children.is_empty() {
            let mut sum = 0;
            for child in self.children.values_mut() {
                child.calculate_sizes();
                sum += child.token_sum;
            }
            // If it's a directory, its sum is the children's sum.
            // Note: If a directory also has own tokens (rare in this model), handled here.
            self.token_sum = sum;
        }
    }
}

/// Helper for Priority Queue
#[derive(Eq, PartialEq)]
struct NodeRef<'a> {
    tokens: usize,
    depth: usize,
    path: &'a str,
}

impl<'a> Ord for NodeRef<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by Tokens (Desc), then Depth (Desc), then Path (Asc)
        self.tokens
            .cmp(&other.tokens)
            .then_with(|| other.depth.cmp(&self.depth)) // deeper first if equal tokens
            .then_with(|| other.path.cmp(self.path)) // reverse path for stability
    }
}

impl<'a> PartialOrd for NodeRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn select_significant_nodes(
    root: &Node,
    total_tokens: usize,
    options: &TokenMapOptions,
    keep_set: &mut HashMap<String, bool>, // Path -> keep
) {
    let mut heap = BinaryHeap::new();
    let min_tokens = (total_tokens as f64 * options.min_percent / 100.0) as usize;

    // Seed heap with root children
    for child in root.children.values() {
        if child.token_sum >= min_tokens {
            heap.push(NodeRef {
                tokens: child.token_sum,
                depth: 0,
                path: &child.path,
            });
        }
    }

    let mut count = 0;
    while let Some(node_ref) = heap.pop() {
        if count >= options.max_lines {
            break;
        }

        keep_set.insert(node_ref.path.to_string(), true);
        count += 1;

        // Find the node in the tree to add its children
        // (Inefficient lookup here but safe. Optimization: traverse and pass refs?
        //  Given depth < 20, string lookup is negligible)
        if let Some(node) = find_node(root, node_ref.path) {
            for child in node.children.values() {
                if child.token_sum >= min_tokens {
                    heap.push(NodeRef {
                        tokens: child.token_sum,
                        depth: node_ref.depth + 1,
                        path: &child.path,
                    });
                }
            }
        }
    }
}

fn find_node<'a>(root: &'a Node, path: &str) -> Option<&'a Node> {
    let mut current = root;
    for part in path.split('/') {
        if part.is_empty() {
            continue;
        }
        current = current.children.get(part)?;
    }
    Some(current)
}

fn flatten_tree(
    node: &Node,
    depth: usize,
    is_last: bool,
    keep_set: &HashMap<String, bool>,
    total_tokens: usize,
    output: &mut Vec<TokenMapEntry>,
    covered_tokens: &mut usize,
) {
    // Only verify non-root nodes
    if !node.path.is_empty() {
        if !keep_set.contains_key(&node.path) {
            return;
        }

        // Check if any children are kept to set `has_children`
        let has_visible_children = node
            .children
            .values()
            .any(|c| keep_set.contains_key(&c.path));

        // Add to output
        output.push(TokenMapEntry {
            path: node.path.clone(),
            name: node.name.clone(),
            tokens: node.token_sum,
            percentage: (node.token_sum as f64 / total_tokens as f64) * 100.0,
            depth: depth.saturating_sub(1), // Adjust because root is depth -1 conceptually
            is_last_child: is_last,
            has_children: has_visible_children,
            metadata: EntryMetadata {
                is_dir: node.is_dir,
            },
        });

        // If it's a file (leaf in display), mark tokens as covered
        if !has_visible_children {
            *covered_tokens += node.token_sum;
        }
    }

    // Sort children by size descending for display
    let mut children: Vec<&Node> = node
        .children
        .values()
        .filter(|c| keep_set.contains_key(&c.path))
        .collect();

    children.sort_by(|a, b| b.token_sum.cmp(&a.token_sum));

    let count = children.len();
    for (i, child) in children.into_iter().enumerate() {
        flatten_tree(
            child,
            if node.path.is_empty() { 0 } else { depth + 1 }, // Reset depth for root children
            i == count - 1,
            keep_set,
            total_tokens,
            output,
            covered_tokens,
        );
    }
}
