//! Pass-through selector: keeps everything. Real include/exclude/size/
//! secret-scan predicate logic lands when the runner needs it; step 2 just
//! needs the trait to have an impl so a spec can name a selector slot.

use crate::pipeline::{RawItem, Selector};

pub struct PassThrough;

impl Selector for PassThrough {
    fn keep(&self, _item: &RawItem) -> bool {
        true
    }
}
