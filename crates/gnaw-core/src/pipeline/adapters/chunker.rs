//! Identity chunker: one item → one chunk. It does NOT split (tree-sitter
//! splitting is a separate, later feature). Its real job in step 2 is
//! resolving `RawContent`'s variants into a single plain `Chunk.text` — the
//! one seam where the diff-vs-file distinction is handled, so no downstream
//! stage has to know about it.

use crate::pipeline::{Chunk, Chunker, RawContent, RawItem};

pub struct IdentityChunker;

impl Chunker for IdentityChunker {
    fn chunk(&self, item: &RawItem) -> Vec<Chunk> {
        let text = match &item.content {
            RawContent::Text { text } => text.clone(),

            // STEP 2 minimal: flatten a change to its current body, or the
            // patch when that's all the mode produced. The FULL per-mode
            // formatting (before/after/patch layout that git-diff-shas.hbs
            // does today) is step-4 work — this chunk's text is recaptured
            // against the step-4 golden, so byte-parity on the changed path
            // is intentionally NOT a step-2 goal.
            RawContent::Changed { after, patch, .. } => {
                if after.is_empty() {
                    // patch-only mode: the change lives in the diff
                    patch.clone().unwrap_or_default()
                } else {
                    after.clone()
                }
            }

            // Binary/over-size: nothing to render, so no chunk.
            RawContent::Omitted => return Vec::new(),
        };

        if text.is_empty() {
            return Vec::new();
        }

        vec![Chunk {
            source_path: item.path.clone(),
            extension: item.extension.clone(),
            text,
            index: 0,
            tokens: 0,
        }]
    }
}
