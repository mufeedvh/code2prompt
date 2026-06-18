//! Port traits. Pure contracts — no git2, ignore, or fs here. Adapters in
//! step 2 implement these by delegating to existing functions.
//!
//! `Send + Sync` on every port: the runner and the future REST/MCP server
//! are multi-threaded, and an axum handler holding a `dyn Renderer` needs it.

use super::*;
use crate::sort::FileSortMethod;

/// Options threaded into a source. Concrete fields land when adapters need
/// them (refs for the commit-range source, root path for the tree source);
/// kept minimal in step 1 so the trait is stable.
#[derive(Debug, Clone, Default)]
pub struct SourceOpts;

/// Yields raw items from somewhere: working tree, commit range, staged, PR.
/// "working-tree files" and "files changed between two refs" are TWO
/// sources, not one source plus a filter — that distinction is the whole
/// reason the changed-files run can avoid walking the tree.
pub trait ContextSource: Send + Sync {
    fn items(&self, opts: &SourceOpts) -> Result<Vec<RawItem>, PipelineError>;
}

/// Include/exclude predicate (globset + size + secret-scan logic).
pub trait Selector: Send + Sync {
    fn keep(&self, item: &RawItem) -> bool;
}

/// Splits an item into chunks. Step-2 ships an identity impl (one chunk =
/// whole file); tree-sitter chunking is a separate, later feature.
pub trait Chunker: Send + Sync {
    fn chunk(&self, item: &RawItem) -> Vec<Chunk>;
}

/// Counts tokens for a target encoding. Wraps tiktoken-rs in step 2. The
/// duplicated count path + `fallback_structural_estimate` is deleted at the
/// end, not ported — that duplication IS the bug.
pub trait TokenCounter: Send + Sync {
    fn count(&self, text: &str) -> usize;
    fn encoding(&self) -> &str;
}

/// Scores a chunk for relevance. No-op (all equal) first.
pub trait Ranker: Send + Sync {
    fn score(&self, chunk: &Chunk, ctx: &RankCtx) -> f32;
}

/// Context handed to the ranker (query string, recency data, …). Empty in
/// step 1; this is NOT a wire type, so no serde — it's an internal handle.
#[derive(Debug, Clone, Default)]
pub struct RankCtx;

/// Packs the highest-value chunks into a token budget. "Take until budget"
/// first. Owns the contract that `Selection.tally.total <= budget` (the
/// property test the doc calls for).
pub trait Budgeter: Send + Sync {
    fn fit(&self, ranked: Vec<ScoredChunk>, budget: usize) -> Selection;
}

/// Pipeline-computed context the renderer needs but the Selection doesn't carry:
/// the source tree (derived from items), the root label, etc. Built inside `run`,
/// NOT supplied by the caller — that's the whole point, since an items-derived
/// tree can't be known before the source runs.
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    pub source_tree: String,
    pub absolute_code_path: String,
}

/// Renders a selection. Wraps handlebars + existing templates in step 2,
/// and owns its own variable contract — replacing the hand-maintained
/// `PROVIDED_IDENTIFIERS` list in `template.rs`.
pub trait Renderer: Send + Sync {
    fn render(&self, sel: &Selection, ctx: &RenderContext) -> Result<Rendered, PipelineError>;
}

/// Builds the source-tree string. Two sourcings: from surviving items (the
/// normal case, output-consistent) or from a full filesystem walk (the
/// `--full-directory-tree` case, which must show filter-dropped paths and so
/// CANNOT derive from items). A stage, not a flag, because "where the tree
/// comes from" is a real axis of variation — a REST request could pick either.
pub trait TreeBuilder: Send + Sync {
    /// `items` is the post-filter set; `root_label`/`sort_method` come from the
    /// spec. A full-walk builder ignores `items` and walks instead.
    fn build(
        &self,
        items: &[RawItem],
        root_label: &str,
        sort_method: Option<FileSortMethod>,
    ) -> String;
}
