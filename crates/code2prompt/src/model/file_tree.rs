//! File tree state management for the TUI application.
//!
//! This module contains the file tree state, FileNode structure,
//! and related functionality for managing the file selection interface.

use std::path::PathBuf;

/// File tree state containing all file tree related data
#[derive(Debug, Clone)]
pub struct FileTreeState {
    pub file_tree: Vec<FileNode>,
    pub search_query: String,
    pub tree_cursor: usize,
    pub file_tree_scroll: u16,
}

/// File tree node with selection state
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub children: Vec<FileNode>,
    pub level: usize,
}

impl Default for FileTreeState {
    fn default() -> Self {
        FileTreeState {
            file_tree: Vec::new(),
            search_query: String::new(),
            tree_cursor: 0,
            file_tree_scroll: 0,
        }
    }
}

impl FileTreeState {
    /// Get flattened list of visible file nodes for display
    pub fn get_visible_nodes(&self) -> Vec<&FileNode> {
        let mut visible = Vec::new();
        self.collect_visible_nodes(&self.file_tree, &mut visible);
        visible
    }

    /// Set the file tree
    pub fn set_file_tree(&mut self, tree: Vec<FileNode>) {
        self.file_tree = tree;
    }

    /// Get a mutable reference to the file tree
    pub fn get_file_tree_mut(&mut self) -> &mut Vec<FileNode> {
        &mut self.file_tree
    }

    fn collect_visible_nodes<'a>(&'a self, nodes: &'a [FileNode], visible: &mut Vec<&'a FileNode>) {
        for node in nodes {
            // Apply search filter - support both simple text and glob patterns
            let matches_search = if self.search_query.is_empty() {
                true
            } else if self.search_query.contains('*') || self.search_query.contains("**") {
                // Treat as glob pattern
                self.glob_match_search(&self.search_query, &node.name)
                    || self.glob_match_search(&self.search_query, &node.path.to_string_lossy())
            } else {
                // Simple text search (case insensitive)
                node.name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
                    || node
                        .path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
            };

            if matches_search {
                visible.push(node);
            }

            // Add children if expanded and node matches search or has matching children
            if node.is_expanded && (matches_search || node.is_directory) {
                self.collect_visible_nodes(&node.children, visible);
            }
        }
    }

    /// Simple glob matching for search (similar to utils but accessible from model)
    fn glob_match_search(&self, pattern: &str, text: &str) -> bool {
        // Handle ** for recursive directory matching
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                if prefix.is_empty() && suffix.is_empty() {
                    return true; // "**" matches everything
                }

                let prefix_match = prefix.is_empty() || text.contains(prefix);
                let suffix_match = suffix.is_empty() || text.contains(suffix);

                return prefix_match && suffix_match;
            }
        }

        // Handle single * wildcard
        if pattern.contains('*') && !pattern.contains("**") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return text.contains(parts[0]) && text.contains(parts[1]);
            }
        }

        // Fallback to contains
        text.to_lowercase().contains(&pattern.to_lowercase())
    }
}

impl FileNode {
    pub fn new(path: PathBuf, level: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_directory = path.is_dir();

        Self {
            path,
            name,
            is_directory,
            is_expanded: false,
            is_selected: false,
            children: Vec::new(),
            level,
        }
    }
}
