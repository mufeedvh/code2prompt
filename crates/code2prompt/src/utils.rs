//! File system utilities and analysis operations.
//!
//! This module provides utilities for building file trees, handling file selection
//! patterns, running code analysis, and managing clipboard/file operations.
//! It bridges the TUI interface with the core code2prompt functionality.

use anyhow::{Context, Result};
use code2prompt_core::session::Code2PromptSession;
use std::fs;

use crate::model::FileNode;

/// Build a file tree using session data from core traversal
pub fn build_file_tree_from_session(session: &mut Code2PromptSession) -> Result<Vec<FileNode>> {
    // Load codebase data using the session
    session
        .load_codebase()
        .context("Failed to load codebase from session")?;

    // Get the files data from session
    let files_data = session
        .data
        .files
        .as_ref()
        .and_then(|f| f.as_array())
        .context("No files data available from session")?;

    // Build a hierarchical tree from session file data
    let mut file_paths = Vec::new();
    for file_entry in files_data {
        if let Some(path_str) = file_entry.get("path").and_then(|p| p.as_str()) {
            file_paths.push(path_str.to_string());
        }
    }

    // Build directory structure
    let mut root_nodes = build_directory_hierarchy(&session.config.path, &file_paths)?;

    // Sort all nodes
    sort_nodes(&mut root_nodes);

    Ok(root_nodes)
}

/// Build directory hierarchy from file paths - simplified approach
fn build_directory_hierarchy(
    root: &std::path::Path,
    file_paths: &[String],
) -> Result<Vec<FileNode>> {
    // For now, fall back to a simple filesystem scan but use session data for selection state
    // This is more reliable than complex hierarchy building
    let entries = fs::read_dir(root).context("Failed to read root directory")?;
    let mut root_children = Vec::new();

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        let mut node = FileNode::new(path, 0);

        // Check if this file/directory should be selected based on session data
        let relative_path = node.path.strip_prefix(root).unwrap_or(&node.path);
        let relative_str = relative_path.to_string_lossy();

        node.is_selected = file_paths.iter().any(|file_path| {
            file_path == &relative_str || file_path.starts_with(&format!("{}/", relative_str))
        });

        // For directories, recursively load if they contain session files
        if node.is_directory {
            let has_session_files = file_paths
                .iter()
                .any(|file_path| file_path.starts_with(&format!("{}/", relative_str)));

            if has_session_files {
                // Load children for this directory since it contains files from session
                if let Ok(children) = build_directory_children(root, &node.path, file_paths, 1) {
                    node.children = children;
                    node.is_expanded = true;
                }
            }
        }

        root_children.push(node);
    }

    Ok(root_children)
}

/// Recursively build children for a directory  
fn build_directory_children(
    root: &std::path::Path,
    dir_path: &std::path::Path,
    file_paths: &[String],
    level: usize,
) -> Result<Vec<FileNode>> {
    if level > 3 {
        return Ok(Vec::new()); // Prevent too deep recursion
    }

    let entries = fs::read_dir(dir_path).context("Failed to read directory")?;
    let mut children = Vec::new();

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        let mut node = FileNode::new(path, level);

        // Check selection against session data
        let relative_path = node.path.strip_prefix(root).unwrap_or(&node.path);
        let relative_str = relative_path.to_string_lossy();

        node.is_selected = file_paths.contains(&relative_str.to_string());

        // Recursively load subdirectories that contain session files
        if node.is_directory {
            let has_session_files = file_paths
                .iter()
                .any(|file_path| file_path.starts_with(&format!("{}/", relative_str)));

            if has_session_files {
                if let Ok(grandchildren) =
                    build_directory_children(root, &node.path, file_paths, level + 1)
                {
                    node.children = grandchildren;
                    node.is_expanded = true;
                }
            }
        }

        children.push(node);
    }

    Ok(children)
}

/// Sort file nodes (directories first, then alphabetically)
fn sort_nodes(nodes: &mut Vec<FileNode>) {
    nodes.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    // Recursively sort children
    for node in nodes {
        sort_nodes(&mut node.children);
    }
}

/// Save text to file
pub fn save_to_file(filename: &str, content: &str) -> Result<()> {
    use code2prompt_core::template::write_to_file;
    write_to_file(filename, content).context("Failed to save to file")
}

/// Get the user's code2prompt data directory following platform conventions
pub fn get_code2prompt_data_dir() -> Result<std::path::PathBuf> {
    let data_dir = if cfg!(target_os = "linux") {
        // Linux: ~/.local/share/code2prompt
        dirs::data_local_dir()
            .context("Failed to get user data directory")?
            .join("code2prompt")
    } else if cfg!(target_os = "windows") {
        // Windows: %APPDATA%/code2prompt
        dirs::data_dir()
            .context("Failed to get user data directory")?
            .join("code2prompt")
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/code2prompt
        dirs::data_dir()
            .context("Failed to get user data directory")?
            .join("code2prompt")
    } else {
        // Fallback for other platforms
        dirs::data_dir()
            .context("Failed to get user data directory")?
            .join("code2prompt")
    };

    // Create the directory if it doesn't exist
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).context("Failed to create code2prompt data directory")?;
    }

    Ok(data_dir)
}

/// Get the default templates directory
pub fn get_code2prompt_default_templates_dir() -> Result<std::path::PathBuf> {
    let default_dir = get_code2prompt_data_dir()?.join("default");

    // Create the directory if it doesn't exist
    if !default_dir.exists() {
        fs::create_dir_all(&default_dir)
            .context("Failed to create code2prompt default templates directory")?;
    }

    Ok(default_dir)
}

/// Get the custom templates directory
pub fn get_code2prompt_custom_templates_dir() -> Result<std::path::PathBuf> {
    let custom_dir = get_code2prompt_data_dir()?.join("custom");

    // Create the directory if it doesn't exist
    if !custom_dir.exists() {
        fs::create_dir_all(&custom_dir)
            .context("Failed to create code2prompt custom templates directory")?;
    }

    Ok(custom_dir)
}

/// Get the user's code2prompt templates directory (legacy)
pub fn get_code2prompt_templates_dir() -> Result<std::path::PathBuf> {
    let templates_dir = get_code2prompt_data_dir()?.join("templates");

    // Create the templates directory if it doesn't exist
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)
            .context("Failed to create code2prompt templates directory")?;
    }

    Ok(templates_dir)
}

/// Save a template to the user's custom templates directory
pub fn save_template_to_custom_dir(filename: &str, content: &str) -> Result<std::path::PathBuf> {
    let custom_dir = get_code2prompt_custom_templates_dir()?;
    let file_path = custom_dir.join(format!("{}.hbs", filename));

    fs::write(&file_path, content)
        .with_context(|| format!("Failed to save template to {}", file_path.display()))?;

    Ok(file_path)
}

/// Load templates from a specific directory
fn load_templates_from_dir(
    dir: &std::path::Path,
    prefix: &str,
) -> Vec<(String, std::path::PathBuf)> {
    let mut templates = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let display_name = if prefix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}: {}", prefix, name.replace(['-', '_'], " "))
                    };
                    templates.push((display_name, path));
                }
            }
        }
    }

    templates
}

/// Load all available templates from default, custom, and built-in locations
pub fn load_all_templates() -> Result<Vec<(String, std::path::PathBuf, bool)>> {
    let mut all_templates = Vec::new();

    // Load default templates
    if let Ok(default_dir) = get_code2prompt_default_templates_dir() {
        let default_templates = load_templates_from_dir(&default_dir, "Default");
        for (name, path) in default_templates {
            all_templates.push((name, path, false));
        }
    }

    // Load custom templates
    if let Ok(custom_dir) = get_code2prompt_custom_templates_dir() {
        let custom_templates = load_templates_from_dir(&custom_dir, "Custom");
        for (name, path) in custom_templates {
            all_templates.push((name, path, false));
        }
    }

    // Load built-in templates from the templates directory in the project
    let builtin_templates_dir = std::path::PathBuf::from("crates/code2prompt-core/templates");
    if builtin_templates_dir.exists() {
        let builtin_templates = load_templates_from_dir(&builtin_templates_dir, "Built-in");
        for (name, path) in builtin_templates {
            all_templates.push((name, path, false));
        }
    }

    // Load legacy user templates
    if let Ok(legacy_templates) = load_user_templates() {
        for (name, path) in legacy_templates {
            let display_name = format!("User: {}", name.replace(['-', '_'], " "));
            all_templates.push((display_name, path, false));
        }
    }

    // Sort templates by name
    all_templates.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(all_templates)
}

/// Load available user templates from the templates directory (legacy)
pub fn load_user_templates() -> Result<Vec<(String, std::path::PathBuf)>> {
    let templates_dir = get_code2prompt_templates_dir()?;
    let mut templates = Vec::new();

    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    templates.push((name.to_string(), path));
                }
            }
        }
    }

    // Sort templates by name
    templates.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(templates)
}
