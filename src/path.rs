//! This module contains the functions for traversing the directory and processing the files.
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use ignore::WalkBuilder;
use serde_json::json;
use termtree::Tree;
use crate::filter::should_include_file;
use log::{debug};

/// Traverses the directory and returns the string representation of the tree and the vector of JSON file representations
pub fn traverse_directory(
    root_path: &PathBuf,
    include: &[String],
    exclude: &[String],
    conflict_include: bool,
    line_number: bool,
    relative_paths: bool,
) -> Result<(String, Vec<serde_json::Value>)> {
    // ~~~ Initialization ~~~
    let mut files = Vec::new();
    let canonical_root_path = root_path.canonicalize()?;
    let parent_directory = label(&canonical_root_path);

    // ~~~ Build the Tree ~~~
    let tree = WalkBuilder::new(&canonical_root_path)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
        .fold(Tree::new(parent_directory.to_owned()), |mut root, entry| {
            let path = entry.path();
            if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) {
                let mut current_tree = &mut root;
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

                // ~~~ Process the file ~~~
                if path.is_file() && should_include_file(path, &include, &exclude, conflict_include) {
                    let code_bytes = fs::read(&path).expect("Failed to read file");
                    let code = String::from_utf8_lossy(&code_bytes);

                    let code_block = wrap_code_block(&code, path.extension().and_then(|ext| ext.to_str()).unwrap_or(""), line_number);

                    if !code.trim().is_empty() && !code.contains(char::REPLACEMENT_CHARACTER) {
                        let file_path = if relative_paths {
                            format!("{}/{}", parent_directory, relative_path.display())
                        } else {
                            path.display().to_string()
                        };

                        files.push(json!({
                            "path": file_path,
                            "extension": path.extension().and_then(|ext| ext.to_str()).unwrap_or(""),
                            "code": code_block,
                        }));
                        debug!("Included file: {}", file_path);

                    } else {
                        debug!("Excluded file (empty or invalid UTF-8): {}", path.display());
                    }
                } else {
                    debug!("Excluded file: {:?}", path.display());
                }
            }

            root
        });

    Ok((tree.to_string(), files))
}


/// Returns the file name or the string representation of the path
pub fn label<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    if path.file_name().is_none() {
        path.to_str().unwrap_or(".").to_owned()
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned()
    }
}

/// Wraps the code block with a delimiter and adds line numbers if required
fn wrap_code_block(code: &str, extension: &str, line_numbers: bool) -> String {
    let delimiter = "`".repeat(3);
    let mut code_with_line_numbers = String::new();
    
    if line_numbers {
        for (line_number, line) in code.lines().enumerate() {
            code_with_line_numbers.push_str(&format!("{:4} | {}\n", line_number + 1, line));
        }
    } else {
        code_with_line_numbers = code.to_string();
    }
    
    format!("{}{}\n{}\n{}", delimiter, extension, code_with_line_numbers, delimiter)
}
