use anyhow::{Result, Context};
use code2prompt_core::{
    configuration::Code2PromptConfig,
    session::Code2PromptSession,
};
use std::path::Path;
use std::fs;

use crate::model::{FileNode, AnalysisResults};

/// Build a file tree from the given configuration
pub fn build_file_tree(config: &Code2PromptConfig) -> Result<Vec<FileNode>> {
    let root_path = &config.path;
    
    if !root_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", root_path.display()));
    }
    
    let mut root_nodes = Vec::new();
    
    // Get immediate children of the root directory
    if root_path.is_dir() {
        let entries = fs::read_dir(root_path)
            .context("Failed to read root directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // Skip hidden files unless configured to include them
            if !config.hidden && is_hidden(&path) {
                continue;
            }
            
            let mut node = FileNode::new(path, 0);
            
            // Check if this file/directory should be selected based on patterns
            node.is_selected = should_include_path(&node.path, config);
            
            // If it's a directory, also load immediate children to check selection
            if node.is_directory {
                if let Ok(children) = load_directory_children_with_config(&node.path, 1, Some(config)) {
                    node.children = children;
                    
                    // Auto-expand if this directory contains selected files (recursively)
                    node.is_expanded = should_auto_expand_directory(&node, config);
                }
            }
            
            root_nodes.push(node);
        }
    } else {
        // Single file
        let mut node = FileNode::new(root_path.to_path_buf(), 0);
        node.is_selected = should_include_path(&node.path, config);
        root_nodes.push(node);
    }
    
    // Sort nodes
    for node in &mut root_nodes {
        node.sort_children();
    }
    
    root_nodes.sort_by(|a, b| {
        // Directories first, then files
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    Ok(root_nodes)
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

/// Check if a path should be included based on configuration patterns
fn should_include_path(path: &Path, config: &Code2PromptConfig) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check exclude patterns first
    for pattern in &config.exclude_patterns {
        if glob_match(pattern, &path_str) {
            return false;
        }
    }
    
    // Check include patterns
    if !config.include_patterns.is_empty() {
        for pattern in &config.include_patterns {
            if glob_match(pattern, &path_str) {
                return true;
            }
        }
        return false; // No include patterns matched
    }
    
    // Default to including if no patterns specified
    true
}

/// Improved glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // Handle ** for recursive directory matching
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');
            
            if prefix.is_empty() && suffix.is_empty() {
                return true; // "**" matches everything
            }
            
            let prefix_match = prefix.is_empty() || text.starts_with(prefix);
            let suffix_match = suffix.is_empty() || text.ends_with(suffix);
            
            return prefix_match && suffix_match;
        }
    }
    
    // Handle single * wildcard
    if pattern.contains('*') && !pattern.contains("**") {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return text.starts_with(parts[0]) && text.ends_with(parts[1]);
        } else if parts.len() > 2 {
            // Multiple wildcards - check sequentially
            let mut current_pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if i == 0 {
                    // First part must match from beginning
                    if !text[current_pos..].starts_with(part) {
                        return false;
                    }
                    current_pos += part.len();
                } else if i == parts.len() - 1 {
                    // Last part must match at end
                    return text[current_pos..].ends_with(part);
                } else {
                    // Middle parts
                    if let Some(pos) = text[current_pos..].find(part) {
                        current_pos += pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            return true;
        }
    }
    
    // Exact match or contains
    if pattern == text {
        true
    } else {
        text.contains(pattern)
    }
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
fn contains_selected_files_recursive(node: &FileNode, config: &Code2PromptConfig) -> bool {
    // Check immediate children
    for child in &node.children {
        if child.is_selected {
            return true;
        }
        
        // If child is a directory, check its contents recursively
        // but only if it has children already loaded to avoid deep scanning
        if child.is_directory && !child.children.is_empty() {
            if contains_selected_files_recursive(child, config) {
                return true;
            }
        }
    }
    
    false
}

/// Run the code2prompt analysis
pub async fn run_analysis(config: Code2PromptConfig) -> Result<AnalysisResults> {
    // Create a session with the configuration
    let mut session = Code2PromptSession::new(config);
    
    // Load the codebase
    session.load_codebase()
        .context("Failed to load codebase")?;
    
    // Handle git operations if enabled
    if session.config.diff_enabled {
        session.load_git_diff()
            .context("Failed to load git diff")?;
    }
    
    if session.config.diff_branches.is_some() {
        session.load_git_diff_between_branches()
            .context("Failed to load git diff between branches")?;
    }
    
    if session.config.log_branches.is_some() {
        session.load_git_log_between_branches()
            .context("Failed to load git log between branches")?;
    }
    
    // Build template data
    let data = session.build_template_data();
    
    // Render the prompt
    let rendered = session.render_prompt(&data)
        .context("Failed to render prompt")?;
    
    let mut file_count = 0;
    
    // Process files from the session data
    if let Some(files) = data.get("files").and_then(|f| f.as_array()) {
        for file_entry in files {
            if let (Some(_path), Some(_lines), Some(_tokens)) = (
                file_entry.get("path").and_then(|p| p.as_str()),
                file_entry.get("lines").and_then(|l| l.as_u64()),
                file_entry.get("token_count").and_then(|t| t.as_u64()),
            ) {
                file_count += 1;
            }
        }
    }
    
    // Generate token map entries if enabled
    let token_map_entries = if rendered.token_count > 0 {
        crate::token_map::generate_token_map_with_limit(
            &data.get("files").and_then(|f| f.as_array()).unwrap_or(&vec![]).to_vec(),
            rendered.token_count,
            Some(50), // Show more entries in TUI
            Some(0.5), // Lower threshold for more detail
        )
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