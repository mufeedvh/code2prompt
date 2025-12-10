//! This module contains the functions for traversing the directory and processing the files.
use crate::configuration::Code2PromptConfig;
use crate::file_processor;
use crate::filter::{build_globset, should_include_file};
use crate::sort::{FileSortMethod, sort_files, sort_tree};
use crate::tokenizer::count_tokens;
use crate::util::strip_utf8_bom;
use anyhow::Result;
use content_inspector::{ContentType, inspect};
use ignore::{WalkBuilder, WalkState};
use log::debug;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
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

/// Represents a file entry with all its metadata and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub extension: String,
    pub code: String,
    pub token_count: usize,
    pub metadata: EntryMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_time: Option<u64>,
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

/// Traverses the directory and returns the string representation of the tree and the vector of file entries.
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
/// * `Result<(String, Vec<FileEntry>)>` - A tuple containing the string representation of the directory
///   tree and a vector of file entries
pub fn traverse_directory(
    config: &Code2PromptConfig,
    selection_engine: Option<&mut crate::selection::SelectionEngine>,
) -> Result<(String, Vec<FileEntry>)> {
    // Phase 1: Discovery - Build tree and collect files to process
    let (tree, files_to_process) = discover_files(config, selection_engine)?;

    // Phase 2: Processing - Process files in parallel
    let mut files = process_files_parallel(files_to_process, config)?;

    // Phase 3: Assembly - Sort and return results
    assemble_results(tree, &mut files, config)
}

// /// Phase 1: Discovery - Walk directories, build tree, and collect files that need processing
// ///
// /// This phase is sequential because:
// /// - Directory walking is already optimized
// /// - Tree building needs sequential structure
// /// - Selection engine has caching that would need synchronization
// fn discover_files(
//     config: &Code2PromptConfig,
//     mut selection_engine: Option<&mut crate::selection::SelectionEngine>,
// ) -> Result<(Tree<String>, Vec<FileToProcess>)> {
//     let canonical_root_path = config.path.canonicalize()?;
//     let parent_directory = display_name(&canonical_root_path);

//     let include_globset = build_globset(&config.include_patterns);
//     let exclude_globset = build_globset(&config.exclude_patterns);

//     // Build the Walker
//     let walker = WalkBuilder::new(&canonical_root_path)
//         .hidden(!config.hidden)
//         .git_ignore(!config.no_ignore)
//         .follow_links(config.follow_symlinks)
//         .build()
//         .filter_map(|entry| entry.ok());

//     // Build the Tree
//     let mut tree = Tree::new(parent_directory.to_owned());
//     let mut files_to_process = Vec::new();

//     for entry in walker {
//         let path = entry.path();
//         if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) {
//             // Use SelectionEngine if available, otherwise fall back to pattern matching
//             let entry_match = if let Some(engine) = selection_engine.as_mut() {
//                 engine.is_selected(relative_path)
//             } else {
//                 should_include_file(relative_path, &include_globset, &exclude_globset)
//             };

//             // Directory Tree
//             let include_in_tree = config.full_directory_tree || entry_match;

//             if include_in_tree {
//                 let mut current_tree = &mut tree;
//                 for component in relative_path.components() {
//                     let component_str = component.as_os_str().to_string_lossy().to_string();
//                     current_tree = if let Some(pos) = current_tree
//                         .leaves
//                         .iter_mut()
//                         .position(|child| child.root == component_str)
//                     {
//                         &mut current_tree.leaves[pos]
//                     } else {
//                         let new_tree = Tree::new(component_str.clone());
//                         current_tree.leaves.push(new_tree);
//                         current_tree.leaves.last_mut().unwrap()
//                     };
//                 }
//             }

//             // Collect files for processing
//             if path.is_file()
//                 && entry_match
//                 && let Ok(metadata) = entry.metadata()
//             {
//                 files_to_process.push(FileToProcess {
//                     absolute_path: path.to_path_buf(),
//                     relative_path: relative_path.to_path_buf(),
//                     metadata,
//                 });
//             }
//         }
//     }

//     Ok((tree, files_to_process))
// }

/// Phase 1: Discovery - Walk directories, build tree, and collect files that need processing
///
/// This phase uses parallel directory walking to efficiently handle IO latency.
fn discover_files(
    config: &Code2PromptConfig,
    selection_engine: Option<&mut crate::selection::SelectionEngine>,
) -> Result<(Tree<String>, Vec<FileToProcess>)> {
    let canonical_root_path = config.path.canonicalize()?;
    let parent_directory = display_name(&canonical_root_path);

    // Prepare shared state for the parallel walker
    let engine_snapshot = selection_engine.as_deref().cloned();
    let include_patterns = config.include_patterns.clone();
    let exclude_patterns = config.exclude_patterns.clone();
    let full_tree = config.full_directory_tree;
    let root_arc = Arc::new(canonical_root_path.clone());

    // Channel for collecting results from threads
    let (tx, rx) = mpsc::channel();

    // Build the Parallel Walker
    let walker = WalkBuilder::new(&canonical_root_path)
        .hidden(!config.hidden)
        .git_ignore(!config.no_ignore)
        .follow_links(config.follow_symlinks)
        .build_parallel();

    // Run the walker
    walker.run({
        let tx_main: mpsc::Sender<(PathBuf, PathBuf, fs::Metadata, bool)> = tx.clone();

        move || {
            let tx = tx_main.clone();
            let root = root_arc.clone();

            // Each thread gets its own clone of the selection logic.
            // This is safe because SelectionEngine is Clone.
            let mut local_engine = engine_snapshot.clone();

            // If no engine is provided, we build globsets once per thread to avoid rebuilding them for every file
            let (inc_glob, exc_glob) = if local_engine.is_none() {
                (
                    Some(build_globset(&include_patterns)),
                    Some(build_globset(&exclude_patterns)),
                )
            } else {
                (None, None)
            };

            Box::new(move |entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };

                let path = entry.path();
                // Calculate relative path
                let relative_path = match path.strip_prefix(&*root) {
                    Ok(p) => p.to_path_buf(),
                    Err(_) => return WalkState::Continue,
                };

                // Determine if the file/dir is selected
                let is_selected = if let Some(eng) = &mut local_engine {
                    eng.is_selected(&relative_path)
                } else {
                    let inc = inc_glob.as_ref().unwrap();
                    let exc = exc_glob.as_ref().unwrap();
                    should_include_file(&relative_path, inc, exc)
                };

                // If we are building a full tree, we send everything.
                // If not, we only send selected items.
                if full_tree || is_selected {
                    if let Ok(metadata) = entry.metadata() {
                        let _ = tx.send((path.to_path_buf(), relative_path, metadata, is_selected));
                    }
                }

                WalkState::Continue
            })
        }
    });

    // Drop the original sender so the receiver knows when to stop
    drop(tx);

    // Collect all results
    let mut entries: Vec<_> = rx.into_iter().collect();

    // Sort entries to ensure deterministic tree structure
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    // Build Tree and Files
    let mut tree = Tree::new(parent_directory);
    let mut files_to_process = Vec::new();

    for (abs_path, rel_path, metadata, is_selected) in entries {
        // Build Directory Tree
        // We traverse down the tree for every entry, finding or creating nodes.
        let mut current_tree = &mut tree;
        for component in rel_path.components() {
            let component_str = component.as_os_str().to_string_lossy().to_string();

            // This is a workaround for the fact that we can't easily hold a mutable reference
            // to a child while iterating. We use indices instead.
            let pos = current_tree
                .leaves
                .iter()
                .position(|child| child.root == component_str);

            if let Some(p) = pos {
                current_tree = &mut current_tree.leaves[p];
            } else {
                let new_tree = Tree::new(component_str.clone());
                current_tree.leaves.push(new_tree);
                current_tree = current_tree.leaves.last_mut().unwrap();
            }
        }

        // Collect files for processing
        if abs_path.is_file() && is_selected {
            files_to_process.push(FileToProcess {
                absolute_path: abs_path,
                relative_path: rel_path,
                metadata,
            });
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
/// - Build FileEntry structures
fn process_files_parallel(
    files_to_process: Vec<FileToProcess>,
    config: &Code2PromptConfig,
) -> Result<Vec<FileEntry>> {
    // Process files in parallel with rayon
    let files: Vec<Option<FileEntry>> = files_to_process
        .par_iter()
        .map(|file_info| process_single_file(file_info, config))
        .collect();

    // Filter out None values (files that failed to process or were empty)
    Ok(files.into_iter().flatten().collect())
}

/// Read file with single-pass binary detection
///
/// Reads file incrementally: first 8KB for binary detection, then remainder if text.
fn read_file_with_binary_check(path: &Path, file_size: u64) -> std::io::Result<Option<Vec<u8>>> {
    const SAMPLE_SIZE: usize = 8192;

    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::with_capacity(file_size.min(1024 * 1024 * 10) as usize); // Cap at 10MB initial allocation

    // Read first chunk for binary detection
    let bytes_to_read = SAMPLE_SIZE.min(file_size as usize);
    let mut sample_buffer = vec![0u8; bytes_to_read];
    file.read_exact(&mut sample_buffer)?;

    // Check if binary
    if inspect(&sample_buffer) == ContentType::BINARY {
        return Ok(None); // Return None for binary files
    }

    // It's text! Add sample to buffer and read the rest
    buffer.extend_from_slice(&sample_buffer);

    // Read remaining bytes if file is larger than sample
    if file_size > SAMPLE_SIZE as u64 {
        file.read_to_end(&mut buffer)?;
    }

    Ok(Some(buffer))
}

/// Process a single file and return its FileEntry representation
fn process_single_file(file_info: &FileToProcess, config: &Code2PromptConfig) -> Option<FileEntry> {
    let path = &file_info.absolute_path;
    let relative_path = &file_info.relative_path;
    let metadata = &file_info.metadata;

    let code_bytes = match read_file_with_binary_check(path, metadata.len()) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            debug!("Skipped binary file: {}", path.display());
            return None;
        }
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

    // Calculate token count if enabled
    let token_count = if config.token_map_enabled {
        count_tokens(&code, &config.encoding)
    } else {
        0
    };

    // Get modification time if date sorting is requested
    let mod_time = if let Some(method) = config.sort_method {
        if method == FileSortMethod::DateAsc || method == FileSortMethod::DateDesc {
            metadata
                .modified()
                .ok()
                .and_then(|mtime| mtime.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
        } else {
            None
        }
    } else {
        None
    };

    debug!(target: "included_files", "Included file: {}", file_path);

    Some(FileEntry {
        path: file_path,
        extension: extension.to_string(),
        code: code_block,
        token_count,
        metadata: EntryMetadata::from(metadata),
        mod_time,
    })
}

/// Phase 3: Assembly - Sort results and return
fn assemble_results(
    mut tree: Tree<String>,
    files: &mut [FileEntry],
    config: &Code2PromptConfig,
) -> Result<(String, Vec<FileEntry>)> {
    // Sort tree and files
    sort_tree(&mut tree, config.sort_method);
    sort_files(files, config.sort_method);

    Ok((tree.to_string(), files.to_owned()))
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
pub fn display_name<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    // File name if available
    if let Some(name) = path.file_name() {
        return name.to_string_lossy().into_owned();
    }
    // Current directory name
    if let Ok(cwd) = std::env::current_dir()
        && let Some(name) = cwd.file_name()
    {
        return name.to_string_lossy().into_owned();
    }
    // Fallback
    ".".to_string()
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
pub fn wrap_code_block(
    code: &str,
    extension: &str,
    line_numbers: bool,
    no_codeblock: bool,
) -> String {
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
