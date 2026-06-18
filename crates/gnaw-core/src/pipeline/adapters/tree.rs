//! Tree builders. `ItemsTree` projects the surviving items (output-consistent,
//! no walk). `FullWalkTree` walks the filesystem for --full-directory-tree,
//! where the tree must include paths the filter dropped — intrinsically a walk,
//! not an items projection.

use crate::configuration::GnawConfig;
use crate::path::{traverse_directory, tree_from_items};
use crate::pipeline::{RawItem, TreeBuilder};
use crate::sort::FileSortMethod;

pub struct ItemsTree;

impl TreeBuilder for ItemsTree {
    fn build(&self, items: &[RawItem], root_label: &str, sort_method: Option<FileSortMethod>) -> String {
        tree_from_items(items, root_label, sort_method)
    }
}

/// Walks the filesystem for the full tree. Holds a config clone because the
/// legacy walk needs the full ignore/hidden/pattern context, not just items.
/// The walk is intrinsic to the feature (show non-output paths), not a
/// double-walk regression — it only runs when --full-directory-tree is set.
pub struct FullWalkTree {
    config: GnawConfig,
}

impl FullWalkTree {
    pub fn new(config: GnawConfig) -> Self {
        Self { config }
    }
}

impl TreeBuilder for FullWalkTree {
    fn build(&self, _items: &[RawItem], _root_label: &str, _sort_method: Option<FileSortMethod>) -> String {
        // traverse_directory already roots the tree at display_name and sorts
        // per config.sort_method, so root_label/sort_method args are redundant
        // here — they're part of the trait for the items builder's sake.
        match traverse_directory(&self.config, None) {
            Ok(t) => t.tree,
            // A failed walk shouldn't abort the whole render; an empty tree
            // degrades gracefully and the file contents still come through.
            Err(_) => String::new(),
        }
    }
}
