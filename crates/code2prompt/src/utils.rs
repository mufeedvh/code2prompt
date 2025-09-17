//! Utility functions for the TUI application.
//!
//! This module contains helper functions for building file trees,
//! managing file operations, and other utility functions used throughout the TUI.

use anyhow::Result;
use code2prompt_core::session::Code2PromptSession;
use std::path::Path;

use crate::model::DisplayFileNode;

/// Build hierarchical file tree from session using traverse_directory with SelectionEngine
pub fn build_file_tree_from_session(
    session: &mut Code2PromptSession,
) -> Result<Vec<DisplayFileNode>> {
    let mut root_nodes = Vec::new();

    // Build root level nodes using ignore crate to respect gitignore
    use ignore::WalkBuilder;
    let walker = WalkBuilder::new(&session.config.path)
        .max_depth(Some(1))
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        if path == session.config.path {
            continue; // Skip root directory itself
        }

        let mut node = DisplayFileNode::new(path.to_path_buf(), 0);

        // Auto-expand recursively if directory contains selected files
        if node.is_directory {
            auto_expand_recursively(&mut node, session);
        }

        root_nodes.push(node);
    }

    // Sort root nodes: directories first, then alphabetically
    root_nodes.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(root_nodes)
}

/// Recursively auto-expand directories that contain selected files
fn auto_expand_recursively(node: &mut DisplayFileNode, session: &mut Code2PromptSession) {
    if !node.is_directory {
        return;
    }

    if directory_contains_selected_files(&node.path, session) {
        node.is_expanded = true;
        // Load children
        if let Err(e) = node.load_children(session) {
            eprintln!("Warning: Failed to load children for {}: {}", node.name, e);
            return;
        }

        // Recursively auto-expand children
        for child in &mut node.children {
            if child.is_directory {
                auto_expand_recursively(child, session);
            }
        }
    }
}

/// Check if a directory contains any selected files (helper function)
fn directory_contains_selected_files(dir_path: &Path, session: &mut Code2PromptSession) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let relative_path = if let Ok(rel) = path.strip_prefix(&session.config.path) {
                rel
            } else {
                continue;
            };

            if session.is_file_selected(relative_path) {
                return true;
            }

            // Recursively check subdirectories
            if path.is_dir() && directory_contains_selected_files(&path, session) {
                return true;
            }
        }
    }
    false
}

/// Get visible nodes for display (flattened tree with search filtering)
pub fn get_visible_nodes(
    nodes: &[DisplayFileNode],
    search_query: &str,
    session: &mut Code2PromptSession,
) -> Vec<DisplayNodeWithSelection> {
    let mut visible = Vec::new();
    collect_visible_nodes_recursive(nodes, search_query, session, &mut visible);
    visible
}

/// Node with selection state for display
#[derive(Debug, Clone)]
pub struct DisplayNodeWithSelection {
    pub node: DisplayFileNode,
    pub is_selected: bool,
}

/// Recursively collect visible nodes
fn collect_visible_nodes_recursive(
    nodes: &[DisplayFileNode],
    search_query: &str,
    session: &mut Code2PromptSession,
    visible: &mut Vec<DisplayNodeWithSelection>,
) {
    for node in nodes {
        let matches_search = if search_query.is_empty() {
            true
        } else {
            node.name
                .to_lowercase()
                .contains(&search_query.to_lowercase())
                || node
                    .path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&search_query.to_lowercase())
        };

        if matches_search {
            let relative_path = if let Ok(rel) = node.path.strip_prefix(&session.config.path) {
                rel
            } else {
                &node.path
            };

            let is_selected = session.is_file_selected(relative_path);

            visible.push(DisplayNodeWithSelection {
                node: node.clone(),
                is_selected,
            });

            // If this is an expanded directory, recursively add its children
            if node.is_directory && node.is_expanded {
                collect_visible_nodes_recursive(&node.children, search_query, session, visible);
            }
        }
    }
}

/// Save content to a file
pub fn save_to_file(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content)?;
    Ok(())
}

/// Save template to custom directory
pub fn save_template_to_custom_dir(filename: &Path, content: &str) -> Result<()> {
    // Create templates directory if it doesn't exist
    let templates_dir = std::env::current_dir()?.join("templates");
    std::fs::create_dir_all(&templates_dir)?;

    let full_path = templates_dir.join(filename);
    std::fs::write(full_path, content)?;
    Ok(())
}

/// Load all available templates (placeholder implementation)
pub fn load_all_templates() -> Result<Vec<(String, String)>> {
    // This is a placeholder - in the real implementation this would
    // scan for template files and return (name, content) pairs
    Ok(vec![(
        "Default".to_string(),
        "Default template content".to_string(),
    )])
}

/// Ensure a path exists in the file tree by creating missing intermediate nodes
pub fn ensure_path_exists_in_tree(
    root_nodes: &mut Vec<DisplayFileNode>,
    target_path: &Path,
    session: &mut Code2PromptSession,
) -> Result<()> {
    let root_path = &session.config.path;

    // Get relative path components
    let relative_path = if let Ok(rel) = target_path.strip_prefix(root_path) {
        rel
    } else {
        return Ok(()); // Path is not under root, nothing to do
    };

    let components: Vec<_> = relative_path.components().collect();
    if components.is_empty() {
        return Ok(());
    }

    // Build path incrementally
    let mut current_path = root_path.to_path_buf();
    let mut current_nodes = root_nodes;

    for (level, component) in components.into_iter().enumerate() {
        current_path.push(component);

        // Find or create node at this level
        let node_name = component.as_os_str().to_string_lossy().to_string();

        // Look for existing node
        let existing_index = current_nodes.iter().position(|n| n.name == node_name);

        if let Some(index) = existing_index {
            // Node exists, ensure it's loaded if it's a directory
            let node = &mut current_nodes[index];
            if node.is_directory && !node.children_loaded {
                let _ = node.load_children(session);
            }
            current_nodes = &mut current_nodes[index].children;
        } else {
            // Node doesn't exist, create it
            let mut new_node = DisplayFileNode::new(current_path.clone(), level);

            if new_node.is_directory {
                let _ = new_node.load_children(session);
            }

            current_nodes.push(new_node);

            // Sort to maintain order
            current_nodes.sort_by(|a, b| match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            });

            // Find the newly inserted node
            let new_index = current_nodes
                .iter()
                .position(|n| n.name == node_name)
                .unwrap();
            current_nodes = &mut current_nodes[new_index].children;
        }
    }

    Ok(())
}
