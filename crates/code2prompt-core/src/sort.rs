//! This module provides sorting methods for files and directory trees.

use serde_json::Value;
use termtree::Tree;

///! Sorting methods for files.

// Define the available sort methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSortMethod {
    NameAsc,  // Sort files alphabetically (A → Z)
    NameDesc, // Sort files alphabetically in reverse (Z → A)
    DateAsc,  // Sort files by modification date (oldest first)
    DateDesc, // Sort files by modification date (newest first)
}

/// Sorts the provided `files` in place using the specified `sort_method`.
///
/// If `sort_method` is `None`, no sorting will be performed.
///
/// # Arguments
///
/// * `files` - A mutable slice of JSON values representing files. Each file is expected
///             to have a `"path"` key (as a string) and a `"mod_time"` key (as a u64).
/// * `sort_method` - An optional `FileSortMethod` indicating how to sort the files.
pub fn sort_files(files: &mut Vec<Value>, sort_method: Option<FileSortMethod>) {
    if let Some(method) = sort_method {
        files.sort_by(|a, b| match method {
            FileSortMethod::NameAsc => {
                let a_path = a.get("path").and_then(Value::as_str).unwrap_or("");
                let b_path = b.get("path").and_then(Value::as_str).unwrap_or("");
                a_path.cmp(b_path)
            }
            FileSortMethod::NameDesc => {
                let a_path = a.get("path").and_then(Value::as_str).unwrap_or("");
                let b_path = b.get("path").and_then(Value::as_str).unwrap_or("");
                b_path.cmp(a_path)
            }
            FileSortMethod::DateAsc => {
                let a_time = a.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                let b_time = b.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                a_time.cmp(&b_time)
            }
            FileSortMethod::DateDesc => {
                let a_time = a.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                let b_time = b.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                b_time.cmp(&a_time)
            }
        });
    }
}

/// Recursively sorts a directory tree (represented by `termtree::Tree<D>`) in place using the specified
/// `FileSortMethod`. For directory nodes, since modification time is typically unavailable, this function
/// falls back to sorting by name. In effect, DateAsc is treated as NameAsc and DateDesc as NameDesc for directories.
///
/// If `sort_method` is `None`, no sorting is performed.
///
/// # Arguments
///
/// * `tree` - A mutable reference to the directory tree.
/// * `sort_method` - An optional `FileSortMethod` that determines the sorting order.
pub fn sort_tree<D: Ord + std::fmt::Display>(
    tree: &mut Tree<D>,
    sort_method: Option<FileSortMethod>,
) {
    if let Some(method) = sort_method {
        // For directories we only have the name (the root), so date-based sorts fall back to name sorting.
        let ascending = match method {
            FileSortMethod::NameAsc | FileSortMethod::DateAsc => true,
            FileSortMethod::NameDesc | FileSortMethod::DateDesc => false,
        };
        sort_tree_impl(tree, ascending);
    }
}

/// Internal helper: recursively sorts the leaves of a directory tree in the specified order.
fn sort_tree_impl<D: Ord + std::fmt::Display>(tree: &mut Tree<D>, ascending: bool) {
    tree.leaves.sort_by(|a, b| {
        if ascending {
            a.root.cmp(&b.root)
        } else {
            b.root.cmp(&a.root)
        }
    });
    for leaf in &mut tree.leaves {
        sort_tree_impl(leaf, ascending);
    }
}
