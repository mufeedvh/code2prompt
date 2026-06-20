//! Pass-through selector: keeps everything. Real include/exclude/size/
//! secret-scan predicate logic lands when the runner needs it; step 2 just
//! needs the trait to have an impl so a spec can name a selector slot.
use crate::filter::FilterEngine;
use crate::pipeline::{RawItem, Selector};
use std::collections::HashSet;
use std::path::Path;

pub struct PassThrough;

impl Selector for PassThrough {
    fn keep(&self, _item: &RawItem) -> bool {
        true
    }
}

pub struct PatternSelector {
    engine: FilterEngine,
}

impl PatternSelector {
    pub fn new(include: &[String], exclude: &[String]) -> Self {
        Self {
            engine: FilterEngine::new(include, exclude),
        }
    }
}

impl Selector for PatternSelector {
    fn keep(&self, item: &RawItem) -> bool {
        // RawItem.path is repo-relative, which is what matches_patterns expects.
        self.engine.matches_patterns(Path::new(&item.path))
    }
}

/// Keeps exactly the items whose path is in an explicit allow-set.
///
/// The TUI's selector. Interactive selection lives in a `SelectionEngine`
/// (globs *plus* per-file toggles with precedence) that `PatternSelector`
/// can't express. The frontend snapshots the engine's resolved selection into
/// repo-relative paths and hands them here, so the pipeline honors exactly what
/// the user ticked. `keep` is `&self`, so the set sidesteps the engine's
/// `&mut self`/cache entirely.
pub struct ExplicitSelector {
    keep: HashSet<String>,
}

impl ExplicitSelector {
    /// `paths` are repo-relative, matching `RawItem.path` from the working-tree
    /// source (built via `strip_prefix` + `to_string_lossy`).
    pub fn new(paths: impl IntoIterator<Item = String>) -> Self {
        Self {
            keep: paths.into_iter().collect(),
        }
    }
}

impl Selector for ExplicitSelector {
    fn keep(&self, item: &RawItem) -> bool {
        self.keep.contains(&item.path)
    }
}
