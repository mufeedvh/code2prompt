//! Utility functions for the TUI application.
//!
//! This module contains helper functions for building file trees,
//! managing file operations, and other utility functions used throughout the TUI.

use crate::model::DisplayFileNode;
use crate::model::{SizeFilter, TokenState};
use anyhow::Result;
use globset::GlobSet;
use gnaw_core::session::GnawSession;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Collect selected leaf files at or under a path (file → itself if selected;
/// directory → walk, respecting ignore/hidden config). Paths are absolute.
pub fn collect_selected_files_under(node_path: &Path, session: &mut GnawSession) -> Vec<PathBuf> {
    let mut out = Vec::new();

    if node_path.is_file() {
        if session.is_file_selected(node_path) {
            out.push(node_path.to_path_buf());
        }
        return out;
    }

    use ignore::WalkBuilder;
    let walker = WalkBuilder::new(node_path)
        .git_ignore(!session.config.no_ignore)
        .hidden(!session.config.hidden)
        .build();

    for entry in walker.flatten() {
        let p = entry.path();
        if p.is_file() && session.is_file_selected(p) {
            out.push(p.to_path_buf());
        }
    }
    out
}

/// Collect every selected leaf in the already-loaded display tree.
/// Selected files are auto-expanded at build time, so their nodes are loaded.
pub fn collect_selected_files_in_tree(
    nodes: &[DisplayFileNode],
    session: &mut GnawSession,
) -> Vec<PathBuf> {
    fn rec(n: &DisplayFileNode, session: &mut GnawSession, out: &mut Vec<PathBuf>) {
        if n.is_directory {
            for c in &n.children {
                rec(c, session, out);
            }
        } else if session.is_file_selected(&n.path) {
            out.push(n.path.clone());
        }
    }
    let mut out = Vec::new();
    for n in nodes {
        rec(n, session, &mut out);
    }
    out
}
/// Build hierarchical file tree from session using traverse_directory with SelectionEngine
pub fn build_file_tree_from_session(session: &mut GnawSession) -> Result<Vec<DisplayFileNode>> {
    let mut root_nodes = Vec::new();

    // Build root level nodes using ignore crate to respect gitignore
    use ignore::WalkBuilder;
    let walker = WalkBuilder::new(&session.config.path)
        .max_depth(Some(1))
        .git_ignore(!session.config.no_ignore) // Respect the no_ignore flag
        .hidden(!session.config.hidden) // Also respect the hidden flag for consistency
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
fn auto_expand_recursively(node: &mut DisplayFileNode, session: &mut GnawSession) {
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
pub(crate) fn directory_contains_selected_files(
    dir_path: &Path,
    session: &mut GnawSession,
) -> bool {
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
    size_filter: Option<SizeFilter>,
    token_states: &HashMap<PathBuf, TokenState>,
    session: &mut GnawSession,
) -> Vec<DisplayNodeWithSelection> {
    let mut visible = Vec::new();
    let search_active = !search_query.is_empty();
    let matcher = build_query_matcher(search_query);
    collect_visible_nodes_recursive(
        nodes,
        &matcher,
        size_filter,
        token_states,
        session,
        &mut visible,
        search_active,
    );
    visible
}

/// A directory's *display* selection is derived from its contents: it shows as
/// selected when at least one leaf beneath it (over the already-loaded children)
/// is selected. Folders carry no selection action of their own under per-file
/// selection, so querying the engine for the directory path would always return
/// the default — this keeps the checkbox honest after bulk/partial (de)selection.
fn dir_is_selected(node: &DisplayFileNode, session: &mut GnawSession) -> bool {
    for child in &node.children {
        if child.is_directory {
            if dir_is_selected(child, session) {
                return true;
            }
        } else if session.is_file_selected(&child.path) {
            return true;
        }
    }
    false
}

/// Matcher for the file-tree search box.
/// - No glob metacharacters (`* ? {`) → case-insensitive substring (interactive default).
/// - Any metacharacter → full glob dialect via the shared build_globset (braces, **, etc.),
///   matched against name and path, same engine as --include/--exclude.
enum QueryMatcher {
    Substr(String),
    Glob(GlobSet),
    MatchAll,
}

fn build_query_matcher(raw: &str) -> QueryMatcher {
    let raw = raw.trim();
    if raw.is_empty() {
        return QueryMatcher::MatchAll;
    }

    let has_glob = raw.contains('*') || raw.contains('?') || raw.contains('{');
    if !has_glob {
        return QueryMatcher::Substr(raw.to_lowercase());
    }

    // Reuse the filter's globset (brace expansion + **/ prefixing + case fold).
    // Lowercase the pattern and match against lowercased text below for case-insensitivity,
    // since globset itself is case-sensitive.
    QueryMatcher::Glob(gnaw_core::filter::build_globset(&[raw.to_lowercase()]))
}

fn matches(m: &QueryMatcher, text: &str) -> bool {
    match m {
        QueryMatcher::MatchAll => true,
        QueryMatcher::Substr(needle) => text.to_lowercase().contains(needle),
        QueryMatcher::Glob(set) => set.is_match(text.to_lowercase()),
    }
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
    matcher: &QueryMatcher,
    size_filter: Option<SizeFilter>,
    token_states: &HashMap<PathBuf, TokenState>,
    session: &mut GnawSession,
    visible: &mut Vec<DisplayNodeWithSelection>,
    search_active: bool,
) {
    for node in nodes {
        let passes_name = if matches!(matcher, QueryMatcher::MatchAll) {
            true
        } else {
            matches(matcher, &node.name) || matches(matcher, &node.path.to_string_lossy())
        };

        // Size filter: files judged by their Done token count; dirs always pass
        // (their visibility comes from surviving descendants). A file with no count
        // can't satisfy a size filter, so it's hidden while a filter is active.
        let passes_size = match (size_filter, node.is_directory) {
            (None, _) => true,
            (Some(_), true) => true,
            (Some(filter), false) => match token_states.get(&node.path) {
                Some(TokenState::Done(n)) => match filter {
                    SizeFilter::GreaterThan(t) => *n > t,
                    SizeFilter::LessThan(t) => *n < t,
                },
                _ => false,
            },
        };
        let matches_current = passes_name && passes_size;

        if search_active {
            // In search mode, traverse into directories regardless of expansion
            let mut child_results: Vec<DisplayNodeWithSelection> = Vec::new();
            if node.is_directory {
                let children = get_children_for_search(node, session);
                collect_visible_nodes_recursive(
                    &children,
                    matcher,
                    size_filter,
                    token_states,
                    session,
                    &mut child_results,
                    true,
                );
            }

            let include_self = matches_current || !child_results.is_empty();

            if include_self {
                let relative_path = if let Ok(rel) = node.path.strip_prefix(&session.config.path) {
                    rel
                } else {
                    &node.path
                };
                let is_selected = if node.is_directory {
                    dir_is_selected(node, session)
                } else {
                    session.is_file_selected(relative_path)
                };

                // Show directories as expanded in search results for better context
                let mut node_clone = node.clone();
                if node_clone.is_directory {
                    node_clone.is_expanded = true;
                }

                visible.push(DisplayNodeWithSelection {
                    node: node_clone,
                    is_selected,
                });

                visible.extend(child_results);
            }
        } else {
            // Normal mode: only include node if it matches (empty query matches all)
            if matches_current {
                let relative_path = if let Ok(rel) = node.path.strip_prefix(&session.config.path) {
                    rel
                } else {
                    &node.path
                };
                let is_selected = if node.is_directory {
                    dir_is_selected(node, session)
                } else {
                    session.is_file_selected(relative_path)
                };

                visible.push(DisplayNodeWithSelection {
                    node: node.clone(),
                    is_selected,
                });

                // Only descend if the directory is expanded
                if node.is_directory && node.is_expanded {
                    collect_visible_nodes_recursive(
                        &node.children,
                        matcher,
                        size_filter,
                        token_states,
                        session,
                        visible,
                        false,
                    );
                }
            }
        }
    }
}

/// Save content to a file
pub fn save_to_file(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content)?;
    Ok(())
}

/// Format a number with thousand separators according to TokenFormat
///
/// - TokenFormat::Raw: returns the number as-is (e.g., "1234567")
/// - TokenFormat::Format: adds separators every 3 digits (e.g., "1,234,567")
///
/// # Arguments
/// * `num` - The number to format
/// * `format` - The token format setting
///
/// # Returns
/// Formatted string representation of the number
pub fn format_number(num: usize, format: &gnaw_core::tokenizer::TokenFormat) -> String {
    use gnaw_core::tokenizer::TokenFormat;

    match format {
        TokenFormat::Raw => num.to_string(),
        TokenFormat::Format => {
            let s = num.to_string();
            let chars: Vec<char> = s.chars().collect();
            let mut result = String::new();

            for (i, c) in chars.iter().enumerate() {
                if i > 0 && (chars.len() - i).is_multiple_of(3) {
                    result.push(',');
                }
                result.push(*c);
            }
            result
        }
    }
}

/// Load children for search mode without mutating the original tree
fn get_children_for_search(
    node: &DisplayFileNode,
    session: &mut GnawSession,
) -> Vec<DisplayFileNode> {
    if !node.is_directory {
        return Vec::new();
    }

    if node.children_loaded {
        return node.children.clone();
    }

    // Load children on the fly without mutating the original tree
    let mut children: Vec<DisplayFileNode> = Vec::new();

    // Use ignore crate to respect gitignore
    use ignore::WalkBuilder;
    let walker = WalkBuilder::new(&node.path)
        .max_depth(Some(1))
        .git_ignore(!session.config.no_ignore) // Respect the no_ignore flag
        .hidden(!session.config.hidden) // Also respect the hidden flag for consistency
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path == node.path {
            continue;
        }

        let mut child = DisplayFileNode::new(path.to_path_buf(), node.level + 1);

        // Auto-expand if contains selected files
        if child.is_directory && directory_contains_selected_files(&child.path, session) {
            child.is_expanded = true;
        }

        children.push(child);
    }

    // Sort children: directories first, then alphabetically
    children.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    children
}

/// Save template to custom directory
pub fn save_template_to_custom_dir(filename: &Path, content: &str) -> Result<()> {
    let templates_dir = if let Some(cfg) = dirs::config_dir() {
        cfg.join("gnaw").join("templates")
    } else {
        // Fallback to current directory if config_dir not available
        std::env::current_dir()?.join("templates")
    };

    std::fs::create_dir_all(&templates_dir)?;
    let full_path = templates_dir.join(filename);
    std::fs::write(full_path, content)?;
    Ok(())
}

/// Find custom templates and return (display_name, absolute_path).
pub fn load_all_templates() -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();

    // Candidate roots
    let mut roots = Vec::new();
    roots.push(std::env::current_dir()?.join("templates"));
    if let Some(cfg) = dirs::config_dir() {
        roots.push(cfg.join("gnaw").join("templates"));
    }

    // Accept common template extensions
    let is_template = |p: &Path| {
        matches!(
            p.extension().and_then(|e| e.to_str()),
            Some("hbs") | Some("handlebars") | Some("md") | Some("tmpl")
        )
    };

    for root in roots {
        if !root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root).min_depth(1).max_depth(2) {
            let entry = entry?;
            let p = entry.path();
            if p.is_file() && is_template(p) {
                let name = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("template")
                    .to_string();
                out.push((
                    name,
                    p.canonicalize()
                        .unwrap_or_else(|_| p.to_path_buf())
                        .to_string_lossy()
                        .into(),
                ));
            }
        }
    }

    // De-duplicate (same path could appear twice)
    // Let the compiler infer tuple types for the sort closure.
    out.sort_by(|a: &(String, String), b: &(String, String)| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    out.dedup_by(|a, b| a.1 == b.1);

    Ok(out)
}

/// Ensure a path exists in the file tree by creating missing intermediate nodes
pub fn ensure_path_exists_in_tree(
    root_nodes: &mut Vec<DisplayFileNode>,
    target_path: &Path,
    session: &mut GnawSession,
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
