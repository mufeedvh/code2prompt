//! Chunker for the changed-files view. Unlike `IdentityChunker`, which
//! flattens a file to its body and DROPS binaries, this one renders the full
//! per-file changed-files block (header + binary notice OR patch/before/after)
//! into a single chunk's text, and KEEPS binaries — the changed-files view
//! reports a changed binary rather than silently omitting it.
//!
//! This is where the per-file layout that `git-diff-shas.hbs` used to own now
//! lives. Presentation in the chunker is consistent with the renderer owning
//! line numbers/fences: the chunk is the unit the template lays out, and for
//! changed files that unit is a pre-formatted block. The changed-files
//! template just iterates these blocks.

use crate::pipeline::{Chunk, Chunker, RawContent, RawItem};
use std::fmt::Write;

pub struct ChangedChunker;

impl Chunker for ChangedChunker {
    fn chunk(&self, item: &RawItem) -> Vec<Chunk> {
        let mut body = String::new();

        // Header: "## path — status (renamed from `old`)". status defaults to
        // "changed" if a source ever leaves it unset (CommitRangeSource always
        // sets it, but the field is Option on the wire).
        let status = item.status.as_deref().unwrap_or("changed");
        let _ = write!(body, "## {} — {}", item.path, status);
        if let Some(old) = &item.old_path {
            let _ = write!(body, " (renamed from `{}`)", old);
        }
        body.push_str("\n\n");

        match &item.content {
            RawContent::Omitted => {
                body.push_str("_Binary file; contents omitted._\n");
            }
            RawContent::Changed {
                after,
                before,
                patch,
            } => {
                if let Some(patch) = patch {
                    let _ = write!(body, "```diff\n{}\n```\n", patch);
                }
                if let Some(before) = before {
                    let _ = write!(body, "### Before\n```\n{}\n```\n", before);
                }
                if !after.is_empty() {
                    let _ = write!(body, "### After\n```\n{}\n```\n", after);
                }
            }
            // A working-tree Text item should never reach this chunker, but if
            // a spec is miswired, render it plainly rather than panic.
            RawContent::Text { text } => {
                let _ = write!(body, "```\n{}\n```\n", text);
            }
        }

        vec![Chunk {
            source_path: item.path.clone(),
            extension: item.extension.clone(),
            text: body,
            index: 0,
            tokens: 0,
        }]
    }
}
