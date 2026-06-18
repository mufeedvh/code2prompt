//! This module contains the functions for traversing the directory and processing the files.
use crate::configuration::GnawConfig;
use crate::file_processor;
use crate::filter::{build_globset, should_include_file};
use crate::secret_scan::{Finding, SCANNER, SecretPolicy, SecretScanner};
use crate::sort::{FileSortMethod, sort_files, sort_tree};
use crate::tokenizer::count_tokens;
use crate::util::strip_utf8_bom;
use anyhow::Result;
use content_inspector::{ContentType, inspect};
use ignore::WalkBuilder;
use log::debug;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
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
pub type SecretFinding = (String, Finding);
pub struct Traversal {
    pub tree: String,
    pub files: Vec<FileEntry>,
    pub findings: Vec<SecretFinding>,
}

/// Raw extraction result for the pipeline source path: unwrapped content,
/// no token count, plus any secret-scan findings. Deliberately distinct from
/// `FileEntry` — the pipeline defers wrapping and counting to later stages,
/// so this carries less than `FileEntry` does.
#[derive(Debug, Clone)]
pub struct RawFile {
    pub path: String,
    pub extension: String,
    pub code: String,
    /// (path, finding) pairs. TEMPORARY home — step 2.5's Scrubber stage
    /// takes ownership of findings and this field goes away.
    pub findings: Vec<(String, Finding)>,
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
    config: &GnawConfig,
    selection_engine: Option<&mut crate::selection::SelectionEngine>,
) -> Result<Traversal> {
    // Phase 1: Discovery - Build tree and collect files to process
    let (tree, files_to_process) = discover_files(config, selection_engine)?;

    // Phase 2: Processing - Process files in parallel
    let (mut files, findings) = process_files_parallel(files_to_process, config)?;

    // Phase 3: Assembly - Sort and return results
    let (tree_str, files) = assemble_results(tree, &mut files, config)?;
    Ok(Traversal {
        tree: tree_str,
        files,
        findings,
    })
}

/// Phase 1: Discovery - Walk directories, build tree, and collect files that need processing
///
/// This phase is sequential because:
/// - Directory walking is already optimized
/// - Tree building needs sequential structure
/// - Selection engine has caching that would need synchronization
fn discover_files(
    config: &GnawConfig,
    mut selection_engine: Option<&mut crate::selection::SelectionEngine>,
) -> Result<(Tree<String>, Vec<FileToProcess>)> {
    let canonical_root_path = config.path.canonicalize()?;
    let parent_directory = display_name(&canonical_root_path);

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
            if path.is_file()
                && entry_match
                && let Ok(metadata) = entry.metadata()
            {
                files_to_process.push(FileToProcess {
                    absolute_path: path.to_path_buf(),
                    relative_path: relative_path.to_path_buf(),
                    metadata,
                });
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
/// - Build FileEntry structures
fn process_files_parallel(
    files_to_process: Vec<FileToProcess>,
    config: &GnawConfig,
) -> Result<(Vec<FileEntry>, Vec<SecretFinding>)> {
    // Process files in parallel with rayon
    let results: Vec<(Option<FileEntry>, Vec<SecretFinding>)> = files_to_process
        .par_iter()
        .map(|fi| process_single_file(fi, config))
        .collect(); // order-preserving, lock-free

    let mut files = Vec::new();
    let mut findings = Vec::new();
    for (entry, fnds) in results {
        if let Some(e) = entry {
            files.push(e);
        }
        findings.extend(fnds);
    }
    // deterministic for snapshots
    findings.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.line.cmp(&b.1.line))
            .then(a.1.rule_id.cmp(b.1.rule_id))
    });
    Ok((files, findings))
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

/// True when `path` matches any of the configured secret-scan allowlist
/// fragments — a simple substring match, so "tests/" skips anything with that
/// segment. Empty list falls back to a sensible built-in set.
fn path_is_allowlisted(path: &str, allow_paths: &[String]) -> bool {
    const DEFAULTS: &[&str] = &[
        "/tests/",
        "/test/",
        "/fixtures/",
        "/testdata/",
        "/__tests__/",
        "_test.",
    ];
    if allow_paths.is_empty() {
        DEFAULTS.iter().any(|frag| path.contains(frag))
    } else {
        allow_paths.iter().any(|frag| path.contains(frag.as_str()))
    }
}

/// Process a single file and return its FileEntry representation
fn process_single_file(
    file_info: &FileToProcess,
    config: &GnawConfig,
) -> (Option<FileEntry>, Vec<(String, Finding)>) {
    let path = &file_info.absolute_path;
    let relative_path = &file_info.relative_path;
    let metadata = &file_info.metadata;
    let no_findings: Vec<(String, Finding)> = Vec::new();

    let code_bytes = match read_file_with_binary_check(path, metadata.len()) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            debug!("Skipped binary file: {}", path.display());
            return (None, no_findings.clone());
        }
        Err(e) => {
            debug!("Failed to read file {}: {}", path.display(), e);
            return (None, no_findings.clone());
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

    // Syntax-aware compression, applied before counting so token totals reflect
    // the compressed output. No-op without the feature, when disabled, or for
    // languages with no grammar.
    #[cfg(feature = "compression")]
    let code = if config.compression.any() {
        match crate::compressor::compressor_for_extension(extension) {
            Some(c) => c.compress(&code, &config.compression),
            None => code,
        }
    } else {
        code
    };

    // Filter empty or invalid files
    if code.trim().is_empty() || code.contains(char::REPLACEMENT_CHARACTER) {
        debug!("Excluded file (empty or invalid UTF-8): {}", path.display());
        return (None, no_findings.clone());
    }

    // Build filepath
    let file_path = if config.absolute_path {
        path.to_string_lossy().to_string()
    } else {
        relative_path.to_string_lossy().to_string()
    };

    let (code, findings): (String, Vec<(String, Finding)>) = if config.secret_scan
        != SecretPolicy::Off
        && !path_is_allowlisted(&file_path, &config.secret_scan_allow_paths)
    {
        let (scrubbed, found) = SCANNER.scrub(&code, config.secret_scan);
        let tagged: Vec<(String, Finding)> =
            found.into_iter().map(|f| (file_path.clone(), f)).collect();
        if config.secret_scan == SecretPolicy::Block && tagged.is_empty() {
            // drop content, but keep findings so the caller can fail loudly
            return (None, tagged);
        }
        (scrubbed, tagged)
    } else {
        (code, Vec::new())
    };

    // Wrap code block
    let code_block = wrap_code_block(&code, config.line_numbers);

    // Always calculate token count in parallel (amortized by I/O wait time)
    // This enables zero-overhead token counting regardless of display preferences
    let token_count = count_tokens(&code, &config.encoding);

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

    (
        Some(FileEntry {
            path: file_path,
            extension: extension.to_string(),
            code: code_block,
            token_count,
            metadata: EntryMetadata::from(metadata),
            mod_time,
        }),
        findings,
    )
}

/// Count tokens for a single file using the same pipeline as full analysis
/// (binary check, BOM strip, extension processor, encoding). Returns None for
/// binary/empty/unreadable files, so callers can render a blank.
pub fn count_file_tokens(path: &Path, config: &GnawConfig) -> Option<usize> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }

    let code_bytes = match read_file_with_binary_check(path, meta.len()) {
        Ok(Some(bytes)) => bytes,
        _ => return None, // binary or read error
    };
    let clean_bytes = strip_utf8_bom(&code_bytes);

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let processor = file_processor::get_processor_for_extension(extension);
    let code = match processor.process(clean_bytes, path) {
        Ok(processed) => processed,
        Err(_) => String::from_utf8_lossy(clean_bytes).into_owned(),
    };

    if code.trim().is_empty() || code.contains(char::REPLACEMENT_CHARACTER) {
        return None;
    }

    Some(count_tokens(&code, &config.encoding))
}

/// Pipeline source extraction: produce *raw* file content for one file —
/// no `wrap_code_block`, no line numbers, no token count. Wrapping and
/// counting are deferred to the renderer and counter stages respectively;
/// that deferral is the point of the migration.
///
/// Reuses the legacy extraction guts (binary check, BOM strip, processor
/// dispatch, compression) and, FOR NOW, the inline secret scrub. The scrub
/// is TEMPORARY: step 2.5 promotes it to a dedicated `Scrubber` stage and
/// this function then yields genuinely unscrubbed content. Until then it
/// matches legacy behavior exactly so no secret can leak through the new
/// path before the stage that guards it exists.
///
/// Returns `None` for binary/empty/unreadable files (the source drops them,
/// same as the legacy traversal). Findings ride out alongside the content so
/// the source adapter can surface them until 2.5 gives them a real home.
pub fn extract_raw_file(
    absolute_path: &Path,
    relative_path: &Path,
    config: &GnawConfig,
) -> Option<RawFile> {
    // (path, extension, raw_code, findings)
    let meta = std::fs::metadata(absolute_path).ok()?;
    if !meta.is_file() {
        return None;
    }

    let code_bytes = match read_file_with_binary_check(absolute_path, meta.len()) {
        Ok(Some(bytes)) => bytes,
        _ => return None, // binary or read error → source drops it
    };
    let clean_bytes = strip_utf8_bom(&code_bytes);

    let extension = absolute_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let processor = file_processor::get_processor_for_extension(extension);
    let code = match processor.process(clean_bytes, absolute_path) {
        Ok(processed) => processed,
        Err(_) => String::from_utf8_lossy(clean_bytes).into_owned(),
    };

    #[cfg(feature = "compression")]
    let code = if config.compression.any() {
        match crate::compressor::compressor_for_extension(extension) {
            Some(c) => c.compress(&code, &config.compression),
            None => code,
        }
    } else {
        code
    };

    if code.trim().is_empty() || code.contains(char::REPLACEMENT_CHARACTER) {
        return None;
    }

    let file_path = if config.absolute_path {
        absolute_path.to_string_lossy().to_string()
    } else {
        relative_path.to_string_lossy().to_string()
    };

    // TEMPORARY (step 2.5 removes this): scrub inline so the new source path
    // is no leakier than the legacy one. The dedicated Scrubber stage will
    // take this over and the source will then yield raw, unscrubbed content.
    let (code, findings): (String, Vec<(String, Finding)>) = if config.secret_scan
        != SecretPolicy::Off
        && !path_is_allowlisted(&file_path, &config.secret_scan_allow_paths)
    {
        let (scrubbed, found) = SCANNER.scrub(&code, config.secret_scan);
        let tagged: Vec<(String, Finding)> =
            found.into_iter().map(|f| (file_path.clone(), f)).collect();
        (scrubbed, tagged)
    } else {
        (code, Vec::new())
    };

    Some(RawFile {
        path: file_path,
        extension: extension.to_string(),
        code,
        findings,
    })
}

/// Phase 3: Assembly - Sort results and return
fn assemble_results(
    mut tree: Tree<String>,
    files: &mut [FileEntry],
    config: &GnawConfig,
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

/// Adds line numbers to a code block if required.
///
/// # Arguments
///
/// * `code` - The code to process.
/// * `line_numbers` - Whether to add line numbers.
///
/// # Returns
///
/// * `String` - The processed code.
pub fn wrap_code_block(code: &str, line_numbers: bool) -> String {
    if line_numbers {
        code.lines()
            .enumerate()
            .map(|(i, line)| format!("{:4} | {}\n", i + 1, line))
            .collect()
    } else {
        code.to_string()
    }
}
