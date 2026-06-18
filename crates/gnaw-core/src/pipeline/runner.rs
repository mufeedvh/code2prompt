//! The runner: the ONE place the stage sequence lives. A frontend builds a
//! `PipelineSpec` naming which adapter fills each slot; the runner threads
//! data stage-to-stage and returns the rendered result. No business logic
//! here — just the wiring order.
//!
//! Source → Filter → Chunk → (Rank) → Budget(+Count) → Render.
//! Counting is internal to the budget stage (the budgeter holds the counter),
//! so the tally is computed once from exactly what's kept.

use super::*;
use crate::path::tree_from_items;

/// Declares which adapter fills each pipeline slot. Trait objects so a
/// frontend composes a spec at runtime (CLI picks a source from flags; a REST
/// handler picks one from a request body). Boxed because sizes differ and the
/// spec outlives any single stack frame in the server case.
pub struct PipelineSpec {
    pub source: Box<dyn ContextSource>,
    pub selector: Box<dyn Selector>,
    pub chunker: Box<dyn Chunker>,
    pub ranker: Box<dyn Ranker>,
    pub budgeter: Box<dyn Budgeter>,
    pub renderer: Box<dyn Renderer>,
    /// 0 = unbudgeted (keep everything), matching the budgeter's convention.
    pub budget: usize,
    /// Root node label for the source tree (use `display_name(&config.path)`).
    pub root_label: String,
    /// Sort order for the items-derived tree; must match the config used to
    /// capture the golden or the default tree ordering drifts.
    pub sort_method: Option<crate::sort::FileSortMethod>,
    pub tree_builder: Box<dyn TreeBuilder>,
}

/// Run the pipeline end to end.
pub fn run(spec: &PipelineSpec, opts: &SourceOpts) -> Result<Rendered, PipelineError> {
    // Source: yield raw items.
    let items = spec.source.items(opts)?;

    // Filter: drop out-of-scope items. Order preserved (determinism).
    let items: Vec<RawItem> = items
        .into_iter()
        .filter(|it| spec.selector.keep(it))
        .collect();

    // ── NEW ── Render context derived from the surviving items. Built HERE,
    // after filtering, so the tree is exactly the set that reaches the output —
    // no separate walk, no binary/empty over-inclusion. This is the double-walk
    // and tree-over-inclusion fix in one place.
    let render_ctx = RenderContext {
        source_tree: spec
            .tree_builder
            .build(&items, &spec.root_label, spec.sort_method),
        absolute_code_path: spec.root_label.clone(),
    };

    // Chunk: each item → 0..n chunks.
    let chunks: Vec<Chunk> = items.iter().flat_map(|it| spec.chunker.chunk(it)).collect();

    // Rank: score each chunk.
    let rank_ctx = RankCtx; // ← was `ctx`, renamed for clarity
    let mut ranked: Vec<ScoredChunk> = chunks
        .into_iter()
        .map(|chunk| {
            let score = spec.ranker.score(&chunk, &rank_ctx);
            ScoredChunk { chunk, score }
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Budget (+count).
    let selection = spec.budgeter.fit(ranked, spec.budget);

    // Render — now takes the items-derived context as a second argument.
    spec.renderer.render(&selection, &render_ctx)
}
