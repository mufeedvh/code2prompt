//! This module provides sorting methods for files and directory trees.

use crate::path::FileEntry;
use serde::{self, Deserialize, Serialize};
use std::fmt;
use termtree::Tree;

// Define the available sort methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileSortMethod {
    /// Sort files by name (A → Z)
    NameAsc,
    /// Sort files by name (Z → A)
    NameDesc,
    /// Sort files by modification date (oldest first)
    DateAsc,
    /// Sort files by modification date (newest first)
    DateDesc,
}

impl fmt::Display for FileSortMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSortMethod::NameAsc => write!(f, "Name (A → Z)"),
            FileSortMethod::NameDesc => write!(f, "Name (Z → A)"),
            FileSortMethod::DateAsc => write!(f, "Date (Old → New)"),
            FileSortMethod::DateDesc => write!(f, "Date (New → Old)"),
        }
    }
}

/// Sorts the provided `files` in place using the specified `sort_method`.
///
/// If `sort_method` is `None`, no sorting will be performed.
///
/// # Arguments
///
/// * `files` - A mutable slice of FileEntry representing files.
/// * `sort_method` - An optional `FileSortMethod` indicating how to sort the files.
pub fn sort_files(files: &mut [FileEntry], sort_method: Option<FileSortMethod>) {
    if let Some(method) = sort_method {
        match method {
            FileSortMethod::NameAsc => {
                files.sort_by(|a, b| a.path.cmp(&b.path));
            }
            FileSortMethod::NameDesc => {
                files.sort_by(|a, b| b.path.cmp(&a.path));
            }
            FileSortMethod::DateAsc => {
                files.sort_by_key(|f| f.mod_time.unwrap_or(0));
            }
            FileSortMethod::DateDesc => {
                files.sort_by_key(|f| std::cmp::Reverse(f.mod_time.unwrap_or(0)));
            }
        }
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
