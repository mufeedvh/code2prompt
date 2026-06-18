//! The runner: the ONE place the stage sequence lives. A frontend builds a
//! `PipelineSpec` naming which adapter fills each slot; the runner threads
//! data stage-to-stage and returns the rendered result. No business logic
//! here — just the wiring order.
//!
//! Source → Filter → Chunk → (Rank) → Budget(+Count) → Render.
//! Counting is internal to the budget stage (the budgeter holds the counter),
//! so the tally is computed once from exactly what's kept.

use super::*;

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

    // Chunk: each item → 0..n chunks. Identity chunker emits 0 for Omitted.
    let chunks: Vec<Chunk> = items
        .iter()
        .flat_map(|it| spec.chunker.chunk(it))
        .collect();

    // Rank: score each chunk. No-op ranker scores all equal, so order is
    // preserved; a real ranker would reorder here.
    let ctx = RankCtx::default();
    let mut ranked: Vec<ScoredChunk> = chunks
        .into_iter()
        .map(|chunk| {
            let score = spec.ranker.score(&chunk, &ctx);
            ScoredChunk { chunk, score }
        })
        .collect();

    // Stable sort by score DESC so a real ranker takes effect, but equal
    // scores (the no-op case) keep source order — byte-stable for goldens.
    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Budget (+count): keep what fits, compute the tally once.
    let selection = spec.budgeter.fit(ranked, spec.budget);

    // Render.
    spec.renderer.render(&selection)
}
