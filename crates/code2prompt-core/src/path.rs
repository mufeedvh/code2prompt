//! This module contains the functions for traversing the directory and processing the files.
use crate::configuration::Code2PromptConfig;
use crate::file_processor;
use crate::filter::{build_globset, should_include_file};
use crate::sort::{FileSortMethod, sort_files, sort_tree};
use crate::tokenizer::count_tokens;
use crate::util::strip_utf8_bom;
use anyhow::Result;
use ignore::WalkBuilder;
use log::debug;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use termtree::Tree;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub is_dir: bool,
    pub is_symlink: bool,
}

impl From<&std::fs::Metadata> for EntryMetadata {
    fn from(meta: &std::fs::Metadata) -> Self {
        Self {
            is_dir: meta.is_dir(),
            is_symlink: meta.is_symlink(),
        }
    }
}

/// Represents a file that needs to be processed
#[derive(Debug, Clone)]
struct FileToProcess {
    /// Absolute path to the file
    absolute_path: PathBuf,
    /// Relative path from the root
    relative_path: PathBuf,
    /// File metadata
    metadata: std::fs::Metadata,
}

/// Traverses the directory and returns the string representation of the tree and the vector of JSON file representations.
///
/// This function uses the provided configuration to determine which files to include, how to format them,
/// and how to structure the directory tree.
///
/// # Arguments
///
/// * `config` - Configuration object containing path, include/exclude patterns, and other settings
/// * `selection_engine` - Optional SelectionEngine for advanced file selection with user actions
///
/// # Returns
///
/// * `Result<(String, Vec<serde_json::Value>)>` - A tuple containing the string representation of the directory
///   tree and a vector of JSON representations of the files
pub fn traverse_directory(
    config: &Code2PromptConfig,
    mut selection_engine: Option<&mut crate::selection::SelectionEngine>,
) -> Result<(String, Vec<serde_json::Value>)> {
    // Phase 1: Discovery - Build tree and collect files to process
    let (tree, files_to_process) = discover_files(config, selection_engine.as_deref_mut())?;

    // Phase 2: Processing - Process files in parallel
    let mut files = process_files_parallel(files_to_process, config)?;

    // Phase 3: Assembly - Sort and return results
    assemble_results(tree, &mut files, config)
}

/// Phase 1: Discovery - Walk directories, build tree, and collect files that need processing
///
/// This phase is sequential because:
/// - Directory walking is already optimized
/// - Tree building needs sequential structure
/// - Selection engine has caching that would need synchronization
fn discover_files(
    config: &Code2PromptConfig,
    mut selection_engine: Option<&mut crate::selection::SelectionEngine>,
) -> Result<(Tree<String>, Vec<FileToProcess>)> {
    let canonical_root_path = config.path.canonicalize()?;
    let parent_directory = label(&canonical_root_path);

    let include_globset = build_globset(&config.include_patterns);
    let exclude_globset = build_globset(&config.exclude_patterns);

    // Build the Walker
    let walker = WalkBuilder::new(&canonical_root_path)
        .hidden(!config.hidden)
        .git_ignore(!config.no_ignore)
        .follow_links(config.follow_symlinks)
        .build()
        .filter_map(|entry| entry.ok());

    // Build the Tree
    let mut tree = Tree::new(parent_directory.to_owned());
    let mut files_to_process = Vec::new();

    for entry in walker {
        let path = entry.path();
        if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) {
            // Use SelectionEngine if available, otherwise fall back to pattern matching
            let entry_match = if let Some(engine) = selection_engine.as_mut() {
                engine.is_selected(relative_path)
            } else {
                should_include_file(relative_path, &include_globset, &exclude_globset)
            };

            // Directory Tree
            let include_in_tree = config.full_directory_tree || entry_match;

            if include_in_tree {
                let mut current_tree = &mut tree;
                for component in relative_path.components() {
                    let component_str = component.as_os_str().to_string_lossy().to_string();
                    current_tree = if let Some(pos) = current_tree
                        .leaves
                        .iter_mut()
                        .position(|child| child.root == component_str)
                    {
                        &mut current_tree.leaves[pos]
                    } else {
                        let new_tree = Tree::new(component_str.clone());
                        current_tree.leaves.push(new_tree);
                        current_tree.leaves.last_mut().unwrap()
                    };
                }
            }

            // Collect files for processing
            if path.is_file() && entry_match {
                if let Ok(metadata) = entry.metadata() {
                    files_to_process.push(FileToProcess {
                        absolute_path: path.to_path_buf(),
                        relative_path: relative_path.to_path_buf(),
                        metadata,
                    });
                }
            }
        }
    }

    Ok((tree, files_to_process))
}

/// Phase 2: Processing - Process files in parallel using rayon
///
/// This phase processes files in parallel:
/// - Read file contents (I/O bound)
/// - Process file content (CPU/I/O bound)
/// - Tokenize if enabled (CPU bound)
/// - Build JSON entries
fn process_files_parallel(
    files_to_process: Vec<FileToProcess>,
    config: &Code2PromptConfig,
) -> Result<Vec<serde_json::Value>> {
    // Process files in parallel with rayon
    let files: Vec<Option<serde_json::Value>> = files_to_process
        .par_iter()
        .map(|file_info| process_single_file(file_info, config))
        .collect();

    // Filter out None values (files that failed to process or were empty)
    Ok(files.into_iter().flatten().collect())
}

/// Process a single file and return its JSON representation
fn process_single_file(
    file_info: &FileToProcess,
    config: &Code2PromptConfig,
) -> Option<serde_json::Value> {
    let path = &file_info.absolute_path;
    let relative_path = &file_info.relative_path;
    let metadata = &file_info.metadata;

    // Read file
    let code_bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            debug!("Failed to read file {}: {}", path.display(), e);
            return None;
        }
    };

    let clean_bytes = strip_utf8_bom(&code_bytes);

    // Get appropriate processor for file extension
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let processor = file_processor::get_processor_for_extension(extension);

    // Process file content
    let code = match processor.process(clean_bytes, path) {
        Ok(processed) => processed,
        Err(e) => {
            log::warn!(
                "File processing failed for {}: {}. Using raw text fallback.",
                path.display(),
                e
            );
            String::from_utf8_lossy(clean_bytes).into_owned()
        }
    };

    // Wrap code block
    let code_block = wrap_code_block(&code, extension, config.line_numbers, config.no_codeblock);

    // Filter empty or invalid files
    if code.trim().is_empty() || code.contains(char::REPLACEMENT_CHARACTER) {
        debug!("Excluded file (empty or invalid UTF-8): {}", path.display());
        return None;
    }

    // Build filepath
    let file_path = if config.absolute_path {
        path.to_string_lossy().to_string()
    } else {
        relative_path.to_string_lossy().to_string()
    };

    // Build File JSON Representation
    let mut file_entry = serde_json::Map::new();
    file_entry.insert("path".to_string(), json!(file_path));
    file_entry.insert("extension".to_string(), json!(extension));
    file_entry.insert("code".to_string(), json!(code_block));

    // Store metadata
    let entry_meta = EntryMetadata::from(metadata);
    if let Ok(meta_value) = serde_json::to_value(entry_meta) {
        file_entry.insert("metadata".to_string(), meta_value);
    }

    // Add token count for the file only if token map is enabled
    if config.token_map_enabled {
        let token_count = count_tokens(&code, &config.encoding);
        file_entry.insert("token_count".to_string(), json!(token_count));
    }

    // If date sorting is requested, record the file modification time
    if let Some(method) = config.sort_method {
        if method == FileSortMethod::DateAsc || method == FileSortMethod::DateDesc {
            let mod_time = metadata
                .modified()
                .ok()
                .and_then(|mtime| mtime.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            file_entry.insert("mod_time".to_string(), json!(mod_time));
        }
    }

    debug!(target: "included_files", "Included file: {}", file_path);

    Some(serde_json::Value::Object(file_entry))
}

/// Phase 3: Assembly - Sort results and return
fn assemble_results(
    mut tree: Tree<String>,
    files: &mut Vec<serde_json::Value>,
    config: &Code2PromptConfig,
) -> Result<(String, Vec<serde_json::Value>)> {
    // Sort tree and files
    sort_tree(&mut tree, config.sort_method);
    sort_files(files, config.sort_method);

    Ok((tree.to_string(), files.clone()))
}

/// Returns the file name or the string representation of the path.
///
/// # Arguments
///
/// * `p` - The path to label.
///
/// # Returns
///
/// * `String` - The file name or string representation of the path.
pub fn label<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    if path.file_name().is_none() {
        let current_dir = std::env::current_dir().unwrap();
        current_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
            .to_owned()
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned()
    }
}

/// Wraps the code block with a delimiter and adds line numbers if required.
///
/// # Arguments
///
/// * `code` - The code block to wrap.
/// * `extension` - The file extension of the code block.
/// * `line_numbers` - Whether to add line numbers to the code.
/// * `no_codeblock` - Whether to not wrap the code block with a delimiter.
///
/// # Returns
///
/// * `String` - The wrapped code block.
fn wrap_code_block(code: &str, extension: &str, line_numbers: bool, no_codeblock: bool) -> String {
    let delimiter = "`".repeat(3);
    let mut code_with_line_numbers = String::new();

    if line_numbers {
        for (line_number, line) in code.lines().enumerate() {
            code_with_line_numbers.push_str(&format!("{:4} | {}\n", line_number + 1, line));
        }
    } else {
        code_with_line_numbers = code.to_string();
    }

    if no_codeblock {
        code_with_line_numbers
    } else {
        format!(
            "{}{}\n{}\n{}",
            delimiter, extension, code_with_line_numbers, delimiter
        )
    }
}
