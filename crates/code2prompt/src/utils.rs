//! File system utilities and analysis operations.
//!
//! This module provides utilities for building file trees, handling file selection
//! patterns, running code analysis, and managing clipboard/file operations.
//! It bridges the TUI interface with the core code2prompt functionality.

use anyhow::{Result, Context};
use code2prompt_core::{
    configuration::Code2PromptConfig,
    session::Code2PromptSession,
};
use std::path::Path;
use std::fs;

use crate::model::{FileNode, AnalysisResults};

/// Build a file tree using session data from core traversal
pub fn build_file_tree_from_session(session: &mut Code2PromptSession) -> Result<Vec<FileNode>> {
    // Load codebase data using the session
    session.load_codebase()
        .context("Failed to load codebase from session")?;
    
    // Get the files data from session
    let files_data = session.data.files.as_ref()
        .and_then(|f| f.as_array())
        .context("No files data available from session")?;
    
    // For now, let's simplify and just create a flat structure with immediate children
    // This avoids the complex hierarchy building while still using session data
    let mut root_nodes = Vec::new();
    
    // Use the core's own directory traversal by doing a simple file system scan
    // but respect the session's include/exclude configuration
    let entries = fs::read_dir(&session.config.path)
        .context("Failed to read root directory")?;
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        // Skip hidden files unless configured to include them
        if !session.config.hidden && is_hidden(&path) {
            continue;
        }
        
        let mut node = FileNode::new(path, 0);
        
        // Check if this file appears in the session data to determine selection
        let relative_path = node.path.strip_prefix(&session.config.path)
            .unwrap_or(&node.path);
        let relative_str = relative_path.to_string_lossy();
        
        // Find if this file is included in the session data
        node.is_selected = files_data.iter().any(|file_entry| {
            if let Some(file_path) = file_entry.get("path").and_then(|p| p.as_str()) {
                file_path == relative_str || file_path.starts_with(&format!("{}/", relative_str))
            } else {
                false
            }
        });
        
        // If it's a directory, load its immediate children
        if node.is_directory {
            if let Ok(children) = load_directory_children_with_config(&node.path, 1, Some(&session.config)) {
                node.children = children;
                
                // Auto-expand if this directory contains selected files
                node.is_expanded = should_auto_expand_directory(&node, &session.config);
            }
        }
        
        root_nodes.push(node);
    }
    
    // Sort all nodes
    sort_nodes(&mut root_nodes);
    
    Ok(root_nodes)
}


/// Sort file nodes (directories first, then alphabetically)
fn sort_nodes(nodes: &mut Vec<FileNode>) {
    nodes.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    // Recursively sort children
    for node in nodes {
        sort_nodes(&mut node.children);
    }
}



/// Load children of a directory with configuration for selection
pub fn load_directory_children_with_config(dir_path: &Path, level: usize, config: Option<&Code2PromptConfig>) -> Result<Vec<FileNode>> {
    let mut children = Vec::new();
    
    if !dir_path.is_dir() {
        return Ok(children);
    }
    
    let entries = fs::read_dir(dir_path)
        .context("Failed to read directory")?;
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        // Skip hidden files if config specifies
        if let Some(cfg) = config {
            if !cfg.hidden && is_hidden(&path) {
                continue;
            }
        }
        
        let mut child_node = FileNode::new(path, level);
        
        // Set selection based on patterns if config is provided
        if let Some(cfg) = config {
            child_node.is_selected = should_include_path(&child_node.path, cfg);
            
            // For directories, only load children for auto-expansion if we're not too deep
            // to avoid performance issues with very deep directory trees
            if child_node.is_directory && level < 3 {
                if let Ok(grandchildren) = load_directory_children_with_config(&child_node.path, level + 1, Some(cfg)) {
                    child_node.children = grandchildren;
                    child_node.is_expanded = should_auto_expand_directory(&child_node, cfg);
                }
            }
        }
        
        children.push(child_node);
    }
    
    // Sort children
    children.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    Ok(children)
}

/// Check if a path should be included - simplified version that just returns true
/// Real filtering logic is handled by core session
fn should_include_path(_path: &Path, _config: &Code2PromptConfig) -> bool {
    // For now, let session handle inclusion logic
    // This is just a placeholder for UI file tree building
    true
}

/// Check if a path is hidden (starts with .)
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Check if a directory should be auto-expanded because it contains selected files
fn should_auto_expand_directory(node: &FileNode, config: &Code2PromptConfig) -> bool {
    if !node.is_directory {
        return false;
    }
    
    // Check if the directory itself is selected
    if node.is_selected {
        return true;
    }
    
    // Recursively check all children and subdirectories
    contains_selected_files_recursive(node, config)
}

/// Recursively check if a directory contains any selected files
fn contains_selected_files_recursive(node: &FileNode, _config: &Code2PromptConfig) -> bool {
    // Check immediate children
    for child in &node.children {
        if child.is_selected {
            return true;
        }
        
        // If child is a directory, check its contents recursively
        // but only if it has children already loaded to avoid deep scanning
        if child.is_directory && !child.children.is_empty() && contains_selected_files_recursive(child, _config) {
            return true;
        }
    }
    
    false
}

/// Run the code2prompt analysis on the configured codebase.
///
/// This function creates a session with the provided configuration, loads the codebase,
/// processes git operations if enabled, renders the prompt template, and returns
/// comprehensive analysis results including token counts and token map data.
///
/// # Arguments
///
/// * `config` - The configuration containing paths, patterns, and analysis options
///
/// # Returns
///
/// * `Result<AnalysisResults>` - Analysis results with file count, token count, prompt, and token map
///
/// # Errors
///
/// Returns an error if the codebase cannot be loaded, git operations fail, or template rendering fails.
pub async fn run_analysis(config: Code2PromptConfig) -> Result<AnalysisResults> {
    // Create a session with the configuration
    let mut session = Code2PromptSession::new(config);
    
    // Use the session's generate_prompt method which handles all the orchestration
    let rendered = session.generate_prompt()
        .context("Failed to generate prompt")?;
    
    let file_count = rendered.files.len();
    
    // Generate token map entries if enabled
    let token_map_entries = if rendered.token_count > 0 {
        if let Some(files_value) = session.data.files.as_ref() {
            if let Some(files_array) = files_value.as_array() {
                crate::token_map::generate_token_map_with_limit(
                    files_array,
                    rendered.token_count,
                    Some(50), // Show more entries in TUI
                    Some(0.5), // Lower threshold for more detail
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(AnalysisResults {
        file_count,
        token_count: Some(rendered.token_count),
        generated_prompt: rendered.prompt,
        token_map_entries,
    })
}

/// Copy text to clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        use crate::clipboard::copy_text_to_clipboard;
        copy_text_to_clipboard(text).context("Failed to copy to clipboard")
    }
    #[cfg(target_os = "linux")]
    {
        // Use the clipboard daemon system for Linux
        use crate::clipboard::spawn_clipboard_daemon;
        spawn_clipboard_daemon(text).context("Failed to spawn clipboard daemon")
    }
}

/// Save text to file
pub fn save_to_file(filename: &str, content: &str) -> Result<()> {
    use code2prompt_core::template::write_to_file;
    write_to_file(filename, content, false).context("Failed to save to file")
}