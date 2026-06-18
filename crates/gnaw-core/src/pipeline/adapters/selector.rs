//! Pass-through selector: keeps everything. Real include/exclude/size/
//! secret-scan predicate logic lands when the runner needs it; step 2 just
//! needs the trait to have an impl so a spec can name a selector slot.
use crate::filter::FilterEngine;
use crate::pipeline::{RawItem, Selector};
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
